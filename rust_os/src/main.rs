#![no_std]
#![no_main]
#![feature(stdsimd)]
#![feature(let_chains)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

mod driver;
mod registers;
mod thread;
mod util;
use core::arch::asm;
use driver::*;
use registers::Registers;
use util::idle;

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

#[link_section = ".init"]
#[no_mangle]
extern "aapcs" fn start() -> ! {
    memory_controller::remap();
    exceptions::IVT::new().init();
    exceptions::AIC::new().init();
    serial::Serial::new().init().enable_interrupts();
    sys_timer::SysTimer::new().init().set_interval(32768); // 1 FPS
    println!("Application start");
    let mut regs = Registers::empty();
    regs.pc = idle as u32;
    unsafe {
        thread::THREADS.create_thread(regs).unwrap();
    }
    idle(); // we just wait for the first timer interrupt
}
