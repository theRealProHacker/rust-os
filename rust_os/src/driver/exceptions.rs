use core::ptr::read_volatile;

use crate::{get_reg, print, println, trampoline, thread, registers::Registers, get_psr, serial, sys_timer, util::wait, set_psr};
use core::arch::asm;
use volatile_register::{WO, RW, RO};

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
            self.data_abort_handler.write(dab_handler);
            self.undef_handler.write(und_handler);
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
        self.set_handler(
            1,
            _src1_handler as u32,
            0,
            SrcType::LowLevelSens,
        );
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


fn thread_function(c: char) {
    for _ in 0..20 {
        println!("{c}");
        wait(500_000);
    }
}

#[naked]
extern "aapcs" fn _src1_handler() {
    trampoline!(src1_handler, 4)
}

#[no_mangle]
extern "aapcs" fn src1_handler(regs: &mut Registers) {
    let timer = sys_timer::SysTimer::new();
    let dbgu = serial::Serial::new();
    // Get a mutable reference to the static THREADS
    let threads = thread::get_threads();
    let thread = threads.get_curr_thread();
    // Save the current state
    thread.regs = regs.clone();
    get_psr!(a = spsr);
    thread.psr = a;
    if timer.status.read() == 1 {
        println!("!");
    } else if dbgu.status.read() & serial::RXRDY != 0 {
        let mut thread_regs = Registers::empty();
        thread_regs.r0 = dbgu.read() as u32;
        thread_regs.pc = thread_function as u32;
        match threads.create_thread(thread_regs) {
            Ok(id) => println!("Created thread {id}"),
            Err(msg) => println!("{msg}"),
        }
    } else {
        println!("unknown interrupt")
    }
    println!("Scheduled next thread: {}", threads.schedule_next());
    let thread = threads.get_curr_thread();
    regs.clone_from(&thread.regs);
    let new_psr = thread.psr;
    set_psr!(spsr = new_psr);
    AIC::new().end_of_interrupt();
}

#[naked]
extern "aapcs" fn dab_handler() {
    trampoline!(_dab_handler, 8)
}

#[no_mangle]
extern "aapcs" fn _dab_handler(regs: &mut Registers) {
    println!("Data Abort at {:x} accessing {:x}", regs.lr, super::memory_controller::get_abort_adress());
    let threads = unsafe { &mut thread::THREADS};
    let id = threads.curr_thread;
    println!("Ending: {:?}", threads.get_thread(id));
    threads.end_thread(id);
    loop {}
}
#[naked]
extern "aapcs" fn und_handler() {
    trampoline!(_und_handler, 4)
}

#[no_mangle]
extern "aapcs" fn _und_handler() {
    let mut a: u32;
    get_reg!(a = lr);
    print!("Undefined Instruction");
    let content = unsafe { read_volatile(((a - 8) - (a % 4)) as *const u32) };
    println!(" at {content:x} ({:x}) ", a - 8);
    loop {}
}

#[naked]
extern "aapcs" fn _swi_handler() {
    trampoline!(swi_handler, 0)
}

#[no_mangle]
extern "aapcs" fn swi_handler(_regs: u32) {
    print!("Software interrupt\n");
}
