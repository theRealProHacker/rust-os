use core::fmt::Write;
use volatile_register::{WO, RW, RO};


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
    fn get_serial() -> &'static mut Serial {
        unsafe {(&mut *(DBGU as *mut Serial)).init()}
    }

    #[inline(always)]
    fn init(&mut self) -> &mut Self {
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

fn get_serial() -> &'static mut Serial {
    unsafe {(&mut *(DBGU as *mut Serial)).init()}
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    Serial::get_serial().write_fmt(args).unwrap();
}

pub fn read() -> u8 {
    Serial::get_serial().read()
}