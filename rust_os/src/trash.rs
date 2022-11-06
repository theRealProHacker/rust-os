
// macro_rules! print {
//     ($($arg:tt)*) => {{
//         write!(serial, $($arg)*);
//     }};
// }

// macro_rules! println {
//     ($($arg:tt)*) => {{
//         writeln!(serial, $($arg)*).unwrap()
//     }};
// }




// trait Print {
//     fn print(&self, serial: Serial, mode: Mode);
// }
// impl Print for u8 {
//     #[inline(always)]
//     fn print(&self, serial: Serial, mode: Mode) {
//         match mode {
//             Mode::C => {
//                 serial.write(*self);
//             },
//             _ => panic!("Invalid mode for u8")
//         }
//     }
// }
// impl Print for &[u8] {
//     #[inline(always)]
//     fn print(&self, serial: Serial, mode: Mode) {
//         match mode {
//             Mode::S => {
//                 serial.write_str(*self);
//             },
//             _ => panic!("Invalid mode for byte string")
//         }
//     }
// }

// impl Print for u32 {
//     #[inline(always)]
//     fn print(&self, serial: Serial, mode: Mode) {
//         match mode {
//             Mode::X => {
//                 const ASCII_DIGIT_OFFSET: u32 = 48;
//                 const ASCII_ALPHA_OFFSET: u32 = 55;
//                 let mut mask: u32 = 0xF0_00_00_00;
//                 for i in 0..8 {
//                     let digit: u32 = (*self & mask) >> ((8-i)*4);
//                     let actual_digit = if digit < 10 {
//                         digit + ASCII_DIGIT_OFFSET
//                     } else {
//                         digit + ASCII_ALPHA_OFFSET
//                     };
//                     serial.write(actual_digit as u8);
//                     mask <<= 4;
//                 }
//                 debug_assert_eq!(mask, 0);
//             },
//             _ => panic!("Invalid mode for u32")
//         }
//     }
// }

// impl <T> Print for &T {
//     #[inline(always)]
//     fn print(&self, serial: Serial, mode: Mode) {
//         match mode {
//             Mode::P => {
//                 let ptr = *self as *const T as u32;
//                 ptr.print(serial, Mode::X);
//             },
//             _ => panic!("Invalid mode for reference")
//         }
//     }
// }