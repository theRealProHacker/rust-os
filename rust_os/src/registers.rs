/// A register struct
#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Registers {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
}

impl Registers {
    /// Generates empty registers initialized at 0
    pub fn empty() -> Registers {
        unsafe { core::mem::transmute([0; 16]) }
    }
}

// TODO: Get Registers for a new thread with up to 4 arguments
