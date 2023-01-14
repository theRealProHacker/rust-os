#![no_std]
#![no_main]
#![feature(stdsimd)]
#![feature(let_chains)]
#![feature(asm_const)]

mod own_asm;
mod exceptions;
mod memory_controller;
mod serial;
mod interrupts;
mod sys_timer;
mod power_management;
// mod thread;
use interrupts::SrcType;
use own_asm::demask_interrupts;
use core::arch::{arm::__nop, global_asm};

global_asm!(include_str!("start.s"), options(raw));

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  println!("\n Panicked: {:?}", info);
  loop {}
}

extern "aapcs" {
  fn _src1_handler();
}

#[link_section = ".init"]
#[no_mangle]
extern "aapcs" fn rust_start() -> ! {
  memory_controller::remap();
  exceptions::IVT::new().init();
  interrupts::AIC::new().set_handler(1, _src1_handler as u32, 0, SrcType::LowLevelSens);
  serial::Serial::new().init().enable_interrupts();
  sys_timer::SysTimer::new().init().set_interval(32768/60); // 60 FPS
  println!("Application start");
  loop {
    unsafe{__nop();}
  }
}

#[no_mangle]
extern "aapcs" fn src1_handler() {
  let timer = sys_timer::SysTimer::new();
  let dbgu = serial::Serial::new();
  if timer.status.read() == 1 {
    println!("!");
  } else if dbgu.status.read() & (serial::RXRDY) != 0 {
    let char = dbgu.read() as char;
    for _ in 1..20 {
      print!("{char}");
    }
  } else {
    println!("unknown interrupt");
  }
  interrupts::AIC::new().end_of_interrupt();
  demask_interrupts();
  loop {}
}