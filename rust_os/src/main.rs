#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

use core::arch::global_asm;
use core::fmt::Write;
use volatile_register::{WO, RW, RO};

// setup
global_asm!(include_str!("start.S"));

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

// consts
const DBGU: u32 = 0xFFFFF200;
const RXEN: u32 = 1 << 4;
const TXEN: u32 = 1 << 6;
const RXRDY: u32 = 1 << 0;
const TXRDY: u32 = 1 << 1;

#[repr(C)]
struct Serial {
    pub control: WO<u32>,
    mode: RW<u32>,
    interrupts: [u32;3],
    pub status: RO<u32>,
    pub receive: RO<u32>,
    pub transmit: WO<u32>,
}

impl Serial {
    #[inline(always)]
    fn init(&mut self) -> &mut Serial {
        unsafe {
            self.control.write(RXEN & TXEN);
        }
        self
    }

    /// Receive ready?
    #[inline(always)]
    fn rx_ready(&self) -> bool {
        (self.status.read() & RXRDY) != 0
    }

    /// Transmit ready?
    #[inline(always)]
    fn tx_ready(&self) -> bool {
        (self.status.read() & TXRDY) != 0
    }

    /// Liest einen char 
    #[inline(always)]
    fn read(&self) -> u8 {
        while !self.rx_ready() {}
        self.receive.read() as u8
    }
    
    /// Schreibt einen char
    #[inline(always)]
    fn write(&self, char: u8) {
        while !self.tx_ready() {}
        unsafe {
            self.transmit.write(char.into());
        }
    }
}

impl Write for Serial {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &char in s.as_bytes() {
            self.write(char);
        }
        Ok(())
    }
}

#[no_mangle]
extern "C" fn cat() {
    let serial: &mut Serial;
    unsafe {
        serial = (&mut *(DBGU as *mut Serial)).init();
    }
    writeln!(serial, "Starting up").unwrap();
    loop {
        let c: u8 = serial.read();
        writeln!(serial, "You typed {}, dec: {c}, hex {c:X}, pointer {:p}", c as char, &c).unwrap();
    }
}