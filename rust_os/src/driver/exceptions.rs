use core::ptr::read;

use crate::{
    println,
    registers::Registers,
    serial, sys_timer,
    thread::{get_threads, State::*, ID},
    trampoline,
    util::{demask_interrupts, idle, mask_interrupts},
};
use core::arch::asm;
use volatile_register::{RO, RW, WO};

const IVT_ADDR: u32 = 0;
const AIC_ADDR: u32 = 0xFFFFF000;

#[repr(C)]
pub struct IVT {
    // Eigentlich RW, aber wir sollten hier nicht lesen
    reset: WO<u32>,
    undef: WO<u32>,
    swi: WO<u32>,
    prefetch: WO<u32>,
    data_abort: WO<u32>,
    reserved: WO<u32>,
    irq: WO<u32>,
    fiq: WO<u32>,
    // Hier sind die pointer zu den echten handlern
    // Wir können nicht unendlich weit springen, deswegen ein Offset von 5*4 = 20(dec) = 1*16+4*1 = 14(hex)
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
        unsafe {
            SWI_VECTORS[0] = exit_handler as u32;
            SWI_VECTORS[1] = fork_handler as u32;
            SWI_VECTORS[2] = sleep_handler as u32;
            SWI_VECTORS[3] = put_char_handler as u32;
            SWI_VECTORS[4] = read_char_handler as u32;
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

#[naked]
extern "aapcs" fn _src1_handler() {
    trampoline!(src1_handler, 4)
}

#[naked]
extern "aapcs" fn _dab_handler() {
    trampoline!(dab_handler, 8)
}

#[naked]
extern "aapcs" fn _und_handler() {
    trampoline!(und_handler, 4)
}

#[naked]
extern "aapcs" fn _swi_handler() {
    trampoline!(swi_handler, 0)
}

/// A function that is called when someone messed up
/// It ends the current user thread
fn exception_fault() {
    let threads = get_threads();
    let id = threads.curr_thread;
    println!("Ending: {:?}", threads.get_thread(id));
    threads.end_thread(id);
}

#[no_mangle]
extern "aapcs" fn src1_handler(regs: &mut Registers) {
    let timer = sys_timer::SysTimer::new();
    let dbgu = serial::Serial::new();
    let threads = get_threads();
    threads.save_state(regs);
    if timer.status.read() == 1 {
        for mut thread in threads.array.iter_mut().filter_map(|&mut x| x) {
            thread.state = match thread.state {
                Sleeping(ms) => {
                    // if we overflow (we go below zero), we set to Ready otherwise we remain Sleeping
                    match ms.checked_sub(ms) {
                        Some(new_ms) => Sleeping(new_ms),
                        None => Ready,
                    }
                }
                other => other,
            }
        }
    } else if dbgu.status.read() & serial::RXRDY != 0 {
        let char = dbgu.read() as u32;
        for mut thread in threads.array.iter_mut().filter_map(|&mut x| x) {
            thread.state = match thread.state {
                WaitingForChar => {
                    thread.regs.r0 = char;
                    Ready
                }
                other => other,
            }
        }
        println!("{threads:#?}");
    } else {
        println!("unknown interrupt")
    }
    println!("Scheduled next thread: {}", threads.schedule_next());
    threads.put_state(regs);
    AIC::new().end_of_interrupt();
}

#[no_mangle]
extern "aapcs" fn dab_handler(regs: &mut Registers) {
    mask_interrupts();
    println!(
        "Data Abort at {:x} accessing {:x}",
        regs.lr,
        super::memory_controller::get_abort_adress()
    );
    exception_fault();
    demask_interrupts();
    idle(); // just wait for timer interrupt
}

#[no_mangle]
extern "aapcs" fn und_handler(regs: &mut Registers) {
    mask_interrupts();
    let a = regs.lr;
    println!("Undefined Instruction at {:x} ({a:x}) ", unsafe {
        read((a - (a % 4)) as *const u32)
    });
    exception_fault();
    demask_interrupts();
    idle();
}

#[derive(Debug)]
pub enum SWICode {
    #[allow(dead_code)]
    Exit,
    Fork,
    Sleep,
    PutChar,
    ReadChar,
}

extern "aapcs" fn exit_handler() {
    let threads = get_threads();
    threads.end_thread(threads.curr_thread);
}

#[allow(improper_ctypes_definitions)]
extern "aapcs" fn fork_handler(regs: &mut Registers) -> Option<ID> {
    get_threads().create_thread(*regs).ok()
}

extern "aapcs" fn sleep_handler(ms: u32) {
    get_threads().curr_mut_thread().state = Sleeping(ms);
}

extern "aapcs" fn put_char_handler(c: u8) {
    println!("{}", c as char);
}

extern "aapcs" fn read_char_handler() {
    get_threads().curr_mut_thread().state = WaitingForChar;
}

static mut SWI_VECTORS: [u32; 5] = [0; 5];

#[no_mangle]
extern "aapcs" fn swi_handler(regs: &mut Registers) {
    mask_interrupts();
    let threads = get_threads();
    threads.save_state(regs);
    // ARM Documentation advises us to read the swi code from the instruction (8 or 24 bit imm)
    let _code = unsafe { read((regs.pc - 4) as *const u32) as u8 };
    println!("swi: {_code}");
    if _code > 4 {
        exception_fault()
    } else {
        unsafe {
            asm!(
                "mov pc, {reg}",
                reg = in(reg) SWI_VECTORS[_code as usize],
                in("r0") regs.r0,
            );
        }
        let code: SWICode = unsafe { core::mem::transmute(_code) };
        println!("Software interrupt: {code:?}");
    }
    threads.put_state(regs);
    demask_interrupts();
}
