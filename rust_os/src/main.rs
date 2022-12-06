#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]

mod own_asm;
mod exceptions;
mod memory_controller;
mod serial;
mod interrupts;
mod sys_timer;
mod power_management;
use core::arch::asm;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

extern "C" fn dab_handler() {
  print!("Data abort");
  loop{}
}

extern "C" fn und_handler() {
  print!("Undefined instruction");
  loop{}
}

extern "C" fn swi_handler() {
  print!("Software interrupt");
  loop{}
}

#[link_section = ".init"]
#[no_mangle]
extern "C" fn _start() {
  memory_controller::remap();
  own_asm::init_sps();
  let dbgu = serial::Serial::new().init();
  println!("Starting up");
  println!("exceptions");
  let ivt = exceptions::IVT::new().init();
  unsafe {
    ivt.data_abort_handler.write(dab_handler as u32);
    ivt.undef_handler.write(und_handler as u32);
    ivt.swi_handler.write(swi_handler as u32);
  }
  // interrupt setup in aic and devices
  println!("interrupts");
  interrupts::AIC::new().set_handler(
    1, src1_trampolin
  );
  println!("debug interrupt enable");
  dbgu.enable_interrupts();
  println!("sys timer");
  let sys_timer = sys_timer::SysTimer::new().init();
  sys_timer.set_interval(32768); // 1 sec
  println!("Application start");
  loop {
    unsafe {
      // reads like "if there is some char in CHAR then (re)set CHAR to None and print char 20 times"
      if let Some(char) = CHAR {
        CHAR = None;
        for _ in 1..20 {
          print!("{char}");
        }
      }
    }
  }
}

static mut CHAR: Option<char> = None;

// #[naked]
pub extern "C" fn src1_trampolin() {
  println!("Debug");
  trampolin!(4, src1_handler);
}

#[inline(never)]
// #[no_mangle]
pub extern "C" fn src1_handler(){
  println!("Debug");
  let timer = sys_timer::SysTimer::new();
  let dbgu = serial::Serial::new();
  if timer.status.read() & 1 != 0 {
    println!("!");
  } else if dbgu.status.read() & (serial::COMMRX) != 0 {
    unsafe {
      CHAR = Some(dbgu.read() as char);
    }
  }
  interrupts::AIC::new().end_of_interrupt();
}