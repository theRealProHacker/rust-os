#![no_std]
#![no_main]
#![feature(stdsimd)]
#![feature(let_chains)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(generic_arg_infer)]

mod consts;
mod driver;
mod thread;
mod user;
mod util;
use consts::*;
use core::arch::asm;
// stuff
use driver::*;
use exceptions::{AIC, IVT};
use memory_controller::remap;
use serial::Serial;
use sys_timer::SysTimer;
use thread::get_threads;
use util::Registers;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    util::mask_interrupts();
    println!("\nPanicked: {info:?}");
    loop {}
}

#[naked]
#[no_mangle]
#[link_section = ".init"]
extern "aapcs" fn _start() {
    unsafe {
        asm!(
            // @ v1 is the moving stack pointer, v2 the moving cpsr
            "mov v1, #{UPPER_STACK}",
            // svc
            "mrs v2, cpsr
            mov sp, v1
            sub v1, #{STACK_SIZE}",
            // undefined
            "bic v2, #{RESET}
            orr v2, #{UND}
            msr CPSR, v2
            mov sp, v1
            sub v1, #{STACK_SIZE}",
            // abort
            "bic v2, #{RESET}
            orr v2, #{ABT}
            msr CPSR, v2
            mov sp, v1
            sub v1, #{STACK_SIZE}",
            // irq
            "bic v2, #{RESET}
            orr v2, #{IRQ}
            msr CPSR, v2
            mov sp, v1
            sub v1, #{STACK_SIZE}",
            // sys & usr
            "orr v2, #{SYS}
            msr CPSR, v2
            mov sp, v1",
            // jump into rust
            "b {start}",
            UPPER_STACK = const KERNEL_MEM,
            STACK_SIZE = const KERNEL_STACK_SIZE,
            UND = const UND_MODE,
            ABT = const ABT_MODE,
            IRQ = const IRQ_MODE,
            SYS = const SYS_MODE,
            RESET = const MODE_RESET,
            start = sym start,
            options(noreturn)
        )
    }
}

extern "aapcs" {
    fn main_thread();
}

#[link_section = ".init"]
extern "aapcs" fn start() -> ! {
    remap();
    IVT::new().init();
    AIC::new().init();
    Serial::new().init().enable_interrupts();
    SysTimer::new().init().set_interval(TIME_SLICE as u16);
    println!("Initialized the sys timer with {MS_PER_SLICE} ms per slice");
    println!("Kernel start");
    // Create the main user thread
    get_threads()
        .init()
        .create_thread(thread!(main_thread()))
        .unwrap();
    util::idle(); // we just wait for the first timer interrupt
}
