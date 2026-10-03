use crate::reg::{read_reg, write_reg};

pub const SYST_CSR: usize = 0xE000_E010;
pub const SYST_RVR: usize = 0xE000_E014;
pub const SYST_CVR: usize = 0xE000_E018;

pub fn init() {
    write_reg(SYST_RVR, 47999);
    write_reg(SYST_CVR, 0);
    write_reg(SYST_CSR, (1 << 0) | (1 << 2));
}

pub fn delay_ms(ms: u32) {
    for _ in 0..ms {
        while (read_reg(SYST_CSR) & (1 << 16)) == 0 {}
    }
}
