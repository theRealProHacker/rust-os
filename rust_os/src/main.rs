#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(stdsimd)]
#![feature(let_chains)]

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
use core::arch::arm::__nop;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  println!("\n Panicked: {:?}", info);
  loop {}
}

#[link_section = ".init"]
#[no_mangle]
extern "C" fn _start() -> ! {
  memory_controller::remap();
  own_asm::init_sps();
  exceptions::IVT::new().init();
  interrupts::AIC::new().set_handler(1, src1_handler as u32, 0, SrcType::LowLevelSens);
  serial::Serial::new().init().enable_interrupts();
  sys_timer::SysTimer::new().init().set_interval(32768/60); // 1 sec
  println!("Application start");
  loop {
    unsafe{__nop();}
  }
}

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