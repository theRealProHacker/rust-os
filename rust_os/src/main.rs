#![no_std]
#![no_main]
#![feature(stdsimd)]
#![feature(let_chains)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(is_some_and)]
// merged main
mod driver;
mod registers;
mod thread;
mod user;
mod util;
use core::arch::asm;
use driver::*;
use registers::Registers;
use util::idle;

use crate::thread::get_threads;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    util::mask_interrupts();
    println!("\n Panicked: {info:?}");
    loop {}
}

#[naked]
#[no_mangle]
#[link_section = ".init"]
extern "aapcs" fn _start() {
    unsafe {
        asm!(
            // @ v1 is the moving stack pointer, v2 the individual stacksizes, v3 the moving cpsr
            "mov v1, #0x24000000
        mov v2, #0x10000 @ 64kB
        @ svc
        mrs v3, cpsr
        mov sp, v1
        sub v1, v2
        @ undefined
        bic v3, #0x1F
        orr v3, #0x1B
        msr CPSR, v3
        mov sp, v1
        sub v1, v2
        @ abort
        bic v3, #0x1F
        orr v3, #0x17
        msr CPSR, v3
        mov sp, v1
        sub v1, v2
        @ irq
        bic v3, #0x1F
        orr v3, #0x12
        msr CPSR, v3
        mov sp, v1
        sub v1, v2
        @ sys & usr
        orr v3, #0x1F
        msr CPSR, v3
        mov sp, v1
        @ jump into rust
        b start",
            options(noreturn)
        )
    }
}

const TIME_SLICE: u32 = 32768;
const MS_PER_SLICE: u32 = TIME_SLICE * 1000 / 32768;

extern "C" {
    fn main_thread();
}

#[link_section = ".init"]
#[no_mangle]
extern "aapcs" fn start() -> ! {
    memory_controller::remap();
    exceptions::IVT::new().init();
    exceptions::AIC::new().init();
    serial::Serial::new().init().enable_interrupts();
    sys_timer::SysTimer::new().init().set_interval(TIME_SLICE);
    println!("Initialized the sys timer with {MS_PER_SLICE} ms per slice");
    println!("Kernel start");
    // Create the idle thread
    let mut iregs = Registers::empty();
    iregs.pc = idle as u32;
    // Create the main thread
    let mut main_regs = Registers::empty();
    main_regs.pc = main_thread as u32;
    get_threads().create_thread(iregs).unwrap();
    get_threads().create_thread(main_regs).unwrap();
    // hack to make idle be the running thread when we first get an interrupt
    get_threads().curr_thread = 0;
    idle(); // we just wait for the first timer interrupt
}
