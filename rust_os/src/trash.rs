macro_rules! set_reg {
  // Wir clearen und dann oren wir
  ($reg:tt, $value: tt) => (
      "bic {set_reg}, {set_reg}, #0x1F",
      concat!("orr ", stringify!($reg), ", {tmp_reg}"),
      tmp_reg = in(reg) $value,
      set_reg = out(reg) _,
  )
}