use core::fmt::Write;
use volatile_register::{WO, RW, RO};

// consts
pub const DBGU_ADDR: u32 = 0xFFFFF200;
pub const RXEN: u32 = 1 << 4;
pub const TXEN: u32 = 1 << 6;
pub const RXRDY: u32 = 1 << 0;
pub const TXRDY: u32 = 1 << 1;
pub const COMMRX: u32 = 1 << 31;

#[repr(C)]
pub struct Serial {
    // p. 330
    pub control: WO<u32>,
    mode: RW<u32>,
    pub int_enable: WO<u32>,
    pub int_disable: WO<u32>,
    pub int_mask: RO<u32>,
    pub status: RO<u32>,
    pub receive: RO<u32>,
    pub transmit: WO<u32>,
}

impl Serial {
    #[inline(always)]
    pub fn new() -> &'static mut Serial {
        unsafe {&mut *(DBGU_ADDR as *mut Serial)}
    }

    #[inline(always)]
    pub fn init(&mut self) -> &mut Self {
        unsafe {
            self.control.write(RXEN | TXEN);
        }
        self
    }

    /// Receive ready?
    #[inline(always)]
    pub fn rx_ready(&self) -> bool {
        (self.status.read() & RXRDY) != 0
    }

    /// Transmit ready?
    #[inline(always)]
    pub fn tx_ready(&self) -> bool {
        (self.status.read() & TXRDY) != 0
    }

    /// Liest einen char 
    #[inline(always)]
    pub fn read(&self) -> u8 {
        while !self.rx_ready() {}
        self.receive.read() as u8
    }
    
    /// Schreibt einen char
    #[inline(always)]
    pub fn write(&self, char: u8) {
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
    Serial::new().write_fmt(args).unwrap();
}

// Synchronous read
// pub fn read() -> u8 {
//     Serial::new().read()
// }