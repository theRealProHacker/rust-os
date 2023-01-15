#![no_std]
#![no_main]
#![feature(stdsimd)]
#![feature(let_chains)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

mod own_asm;
mod exceptions;
mod memory_controller;
mod serial;
mod interrupts;
mod sys_timer;
mod power_management;
mod thread;
mod util;
use util::{wait, idle, exit};
use thread::Registers;
use core::arch::{arm::__nop, global_asm};

global_asm!(include_str!("start.s"), options(raw));

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  own_asm::mask_interrupts();
  println!("\n Panicked\nmsg: {:?}\npayload: {:?}", info.message(), info.payload());
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
  interrupts::AIC::new().set_handler(1, _src1_handler as u32, 0, interrupts::SrcType::LowLevelSens);
  serial::Serial::new().init().enable_interrupts();
  sys_timer::SysTimer::new().init().set_interval(32768); // 1 FPS
  println!("Application start");
  loop {
    unsafe{__nop();}
  }
}

fn thread_function(c: char) {
  for _ in 0..20 {
    println!("{c}");
    wait(50000);
  }
  exit();
}

#[no_mangle]
extern "aapcs" fn src1_handler(regs: &mut thread::Registers) {
  let timer = sys_timer::SysTimer::new();
  let dbgu = serial::Serial::new();
  // Get a mutable reference to the static THREADS
  #[allow(non_snake_case)]
  let THREADS = unsafe{&mut thread::THREADS};
  match THREADS.get_curr_thread() {
    Some(thread) => thread.regs = regs.clone(),
    None => ()
  };
  if timer.status.read() == 1 {
    println!("!");
  } else if dbgu.status.read() & (serial::RXRDY) != 0 {
    let mut thread_regs = Registers::empty();
    thread_regs.r0 = dbgu.read() as u32;
    thread_regs.pc = thread_function as u32;
    match THREADS.create_thread(thread_regs) {
      Ok(id) => println!("Created thread {id}"),
      Err(msg) => println!("{msg}")
    }
  } else { println!("unknown interrupt") }
  match THREADS.schedule_next() {
    Ok(id) => println!("Scheduled next thread: {id}"),
    Err(msg) => println!("{msg}")
  }
  match THREADS.get_curr_thread() {
    Some(thread) => {
      println!("Stack pointer to Registers: {regs:p}");
      regs.clone_from(&thread.regs);
      println!("Saved regs: {:?}", thread.regs);
      println!("Stack regs: {regs:?} at {regs:p}");
      interrupts::AIC::new().end_of_interrupt();
    },
    None => {
      interrupts::AIC::new().end_of_interrupt();
      idle()
    }
  }
}