use volatile_register::WO;

const PIOB: u32 = 0xfffff600;
const YELLOW_LED: u32 = 1 << 27;

// Hier nutzen wir eine C-ähnliche Implementation mit Structs
// Die Register werden als Write-Only abstrahiert, so wie es auch in der Dokumentation steht.
#[repr(C)]
struct LED {
    pub per: WO<u32>,
    unused0: [u32; 3],
    pub oer: WO<u32>,
    unused1: [u32; 7],
    pub sodr: WO<u32>,
    pub codr: WO<u32>,
}

impl LED {
    fn new() -> &'static mut LED {
        unsafe { &mut *(PIOB as *mut LED) }
    }

    fn init(&mut self) -> &mut Self {
        unsafe {
            self.per.write(YELLOW_LED);
            self.oer.write(YELLOW_LED);
        }
        self
    }

    fn on(&mut self) {
        unsafe {
            self.sodr.write(YELLOW_LED);
        }
    }

    fn off(&mut self) {
        unsafe {
            self.codr.write(YELLOW_LED);
        }
    }
}

#[no_mangle]
extern "C" fn yellow_on() {
    let led = LED::new().init();
    loop {
        led.on();
        crate::util::wait(100_000);
        led.off();
        crate::util::wait(100_000);
    }
}
