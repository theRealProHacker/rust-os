//! Diese Datei beschreibt die exception handler und deren Initialisierung auf der Hardware

use crate::{
    get_psr,
    memory_controller::get_abort_adress,
    println, serial,
    serial::Serial,
    sys_timer::SysTimer,
    thread::{get_threads, State::*, ThreadList},
    trampoline,
    util::{demask_interrupts, mask_interrupts},
    Registers, USR_MODE,
};
use core::{arch::asm, mem, ptr::read};
use volatile_register::{RO, RW, WO};

const IVT_ADDR: u32 = 0;
const AIC_ADDR: u32 = 0xFFFFF000;

#[repr(C)]
pub struct IVT {
    reset: WO<u32>,
    undef: WO<u32>,
    swi: WO<u32>,
    prefetch: WO<u32>,
    data_abort: WO<u32>,
    reserved: WO<u32>,
    irq: WO<u32>,
    fiq: WO<u32>,
    pub undef_handler: WO<extern "aapcs" fn()>,
    pub swi_handler: WO<extern "aapcs" fn()>,
    prefetch_handler: WO<extern "aapcs" fn()>,
    pub data_abort_handler: WO<extern "aapcs" fn()>,
}

impl IVT {
    #[inline(always)]
    pub fn new() -> &'static mut Self {
        unsafe { &mut *(IVT_ADDR as *mut Self) }
    }

    #[inline(always)]
    pub fn init(&mut self) -> &mut Self {
        unsafe {
            // AT91_interrupts.pdf p.2
            // https://armconverter.com/?code=ldr%20pc,%5Bpc,%23-0xF20%5D
            self.irq.write(0xE51FFF20);
            // Hier müssen wir in die Register den assembly code reinschreiben,
            // der in den handler springt, der in den handler-Registern steht
            // https://armconverter.com/?code=ldr%20pc,%20%5Bpc,%20%230x14%5D
            const ASM_AS_BYTES: u32 = 0xE59FF014;
            self.undef.write(ASM_AS_BYTES);
            self.swi.write(ASM_AS_BYTES);
            // self.prefetch.write(asm_as_bytes); Not needed
            self.data_abort.write(ASM_AS_BYTES);
        }
        unsafe {
            self.data_abort_handler.write(_dab_handler);
            self.undef_handler.write(_und_handler);
            self.swi_handler.write(_swi_handler);
        }
        self
    }
}

#[allow(dead_code)]
pub enum SrcType {
    LowLevelSens,
    NegativeEdgeTriggered,
    HighLevelSens,
    PositiveEdgeTriggered,
}

pub struct AIC {
    // p. 251
    pub src_modes: [RW<u32>; 32],
    pub src_vctrs: [RW<u32>; 32],
    pub ivr: RO<u32>,
    pub fvr: RO<u32>,
    _isr: u32,
    _ipr: u32,
    _imr: u32,
    _unused0: [u32; 3],
    pub enable: WO<u32>,
    _unused1: [u32; 3],
    pub eoicr: WO<u32>,
}

impl AIC {
    #[inline(always)]
    pub fn new() -> &'static mut AIC {
        unsafe { &mut *(AIC_ADDR as *mut AIC) }
    }

    #[inline(always)]
    pub fn init(&mut self) {
        self.set_handler(1, _src1_handler as u32, 0, SrcType::LowLevelSens);
    }

    #[inline(always)]
    pub fn enable_interrupt(&mut self, index: u8) {
        unsafe {
            self.enable.write(1 << index);
        }
        crate::util::demask_interrupts();
    }

    /// Setzt den handler an [index] mit Priorität [prio] und Source Typ [src_type]
    /// index muss zwischen 0 und 31 sein.
    /// prio muss zwischen 0 und 7 sein
    #[inline(always)]
    pub fn set_handler(
        &mut self,
        index: usize,
        handler: u32,
        prio: u32,
        src_type: SrcType,
    ) -> &mut Self {
        unsafe {
            self.src_modes[index].write(prio | ((src_type as u32) << 5));
            self.src_vctrs[index].write(handler);
        }
        self.enable_interrupt(index as u8);
        self
    }

    /// Must be called after interrupt completion
    #[inline(always)]
    pub fn end_of_interrupt(&mut self) {
        unsafe { self.eoicr.write(1) }
        crate::util::demask_interrupts();
    }
}

trampoline! {_src1_handler=>src1_handler@4}
trampoline! {_dab_handler=>dab_handler@8}
trampoline! {_und_handler=>und_handler@4}
trampoline! {_swi_handler=>swi_handler@0}

#[inline(always)]
fn end_handler(regs: &mut Registers) {
    get_threads().put_state(regs);
    demask_interrupts();
}

fn timer_handler(threads: &mut ThreadList) {
    for thread in threads.array.iter_mut().filter_map(|x| x.as_mut()) {
        match thread.state {
            Sleeping(ms) => {
                // if we overflow (we go below zero), we set to Ready otherwise we remain Sleeping
                thread.state = match ms.checked_sub(ms) {
                    Some(new_ms) => Sleeping(new_ms),
                    None => Ready,
                }
            }
            _ => (),
        }
    }
    println!("!");
}

fn dbgu_handler(threads: &mut ThreadList) {
    let dbgu = Serial::new();
    let char = dbgu.read() as u32;
    if char == 4 {
        // ctrl+d is debug print
        println!("{threads:#?}");
        return;
    }
    for thread in threads.iter_mut() {
        match thread.state {
            WaitingForChar => {
                thread.regs.r0 = char;
                thread.state = Ready;
            }
            _ => (),
        }
    }
}

extern "aapcs" fn src1_handler(regs: &mut Registers) {
    let threads = get_threads();
    threads.save_state(regs);
    println!("Interrupt");
    if SysTimer::new().status.read() == 1 {
        timer_handler(threads);
    } else if Serial::new().status.read() & serial::RXRDY != 0 {
        dbgu_handler(threads);
    } else {
        println!("unknown interrupt")
    }
    threads.schedule_next();
    threads.put_state(regs);
    AIC::new().end_of_interrupt();
}

/// A function that is called when someone messed up
/// It ends the current user thread
#[inline(always)]
fn exception_fault() {
    get_psr!(psr = spsr);
    let mode = psr & crate::MODE_RESET;
    if mode == USR_MODE {
        let threads = get_threads();
        let id = threads.curr_thread;
        println!(
            "Exception Fault in User Mode: Ending the current thread: {:?}",
            threads.get_thread(id)
        );
        threads.end_thread(id);
    } else {
        panic!("Exception Fault while in mode {:?}", crate::show_mode(mode))
    }
}

extern "aapcs" fn dab_handler(regs: &mut Registers) {
    mask_interrupts();
    println!(
        "Data Abort at {:x} accessing {:x}",
        regs.pc,
        get_abort_adress()
    );
    exception_fault();
    end_handler(regs);
}

extern "aapcs" fn und_handler(regs: &mut Registers) {
    mask_interrupts();
    println!("Undefined Instruction at {:x}", regs.pc);
    exception_fault();
    end_handler(regs);
}

const SWI_CODE_NUM: usize = 5;

#[derive(Debug)]
pub enum SWICode {
    Exit,
    Fork,
    Sleep,
    PutChar,
    ReadChar,
}

impl From<u8> for SWICode {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute(value) }
    }
}

extern "aapcs" fn swi_handler(regs: &mut Registers) {
    mask_interrupts();
    let threads = get_threads();
    threads.save_state(regs);
    // ARM Documentation advises us to read the swi code from the instruction (8 bit imm)
    let _code = unsafe { read((regs.pc - 4) as *const u8) };
    if _code >= SWI_CODE_NUM as u8 {
        return end_handler(regs);
    }
    let code = SWICode::from(_code);
    use SWICode::*;
    match code {
        Exit => _ = threads.end_thread(threads.curr_thread),
        // Fork gibt C-like bei einem Fehler 0 zurück, da ja der idle-thread immer die id 0 hat.
        // Dadurch kann kein neuer Thread jemals diese ID haben.
        // Wir nutzen diesen Fakt, um die Größe des Rückgabewertes auf usize = u32 zu minimieren
        Fork => {
            regs.r0 = {
                let regs = unsafe { read(regs.r0 as *const Registers) };
                get_threads().create_thread(regs).unwrap_or_else(|err| {
                    println!("Error in Fork handler: {err}");
                    0
                }) as u32
            }
        }
        Sleep => get_threads().curr_mut_thread().state = Sleeping(regs.r0),
        PutChar => Serial::new().write(regs.r0 as u8),
        ReadChar => get_threads().curr_mut_thread().state = WaitingForChar,
    }
    threads.schedule_next();
    end_handler(regs);
}
