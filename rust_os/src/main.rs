#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(const_mut_refs)]
#![feature(asm_const)]

mod own_asm;
mod exceptions;
mod memory_controller;
mod serial;
mod interrupts;
mod sys_timer;
mod power_management;
use core::{arch::asm, ptr::read_volatile};
use interrupts::SrcType;
use own_asm::demask_interrupts;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

extern "aapcs" fn dab_handler() {
  let a: u32;
  get_reg!(a=lr);
  let content = unsafe {
    read_volatile((a-8) as *const [u32;16])
  };
  println!("Data abort at {} with context: {content:?}", a-8);
  loop{}
}

extern "aapcs" fn und_handler() {
  let a: u32;
  get_reg!(a=lr);
  let content = unsafe {
    read_volatile((a-8) as *const [u32;16])
  };
  println!("Undefined Instruction at {} with context: {content:?}", a-8);
  loop{}
}

extern "aapcs" fn swi_handler() {
  trampolin!(0, _swi_handler);
}

fn _swi_handler() {
  print!("Software interrupt\n");
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
    ivt.data_abort_handler.write(dab_handler);
    ivt.undef_handler.write(und_handler);
    ivt.swi_handler.write(swi_handler);
  }
  // interrupt setup in aic and devices
  println!("interrupts");
  interrupts::AIC::new().set_handler(
    1, 
    src1_handler as u32, 
    0, 
    SrcType::LowLevelSens
  );
  println!("debug interrupt enable");
  dbgu.enable_interrupts();
  println!("sys timer");
  let sys_timer = sys_timer::SysTimer::new().init();
  sys_timer.set_interval(32768/2); // 1 sec
  // let c = read(); 
  // if c == b's' {
  //   unsafe {
  //     asm!("swi 0")
  //   }
  // }
  println!("Application start");
  loop {
    unsafe {
      asm!("nop");
    }
  }
}

// static mut CHAR: Option<char> = None;

extern "aapcs" fn src1_handler() {
//   trampolin!(0, _src1_handler);
// }

// #[inline(never)]
// extern "aapcs" fn _src1_handler(){
  let timer = sys_timer::SysTimer::new();
  let dbgu = serial::Serial::new();
  if timer.status.read() & 1 != 0 {
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