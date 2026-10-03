use crate::nxp::gpio::{GPIO4_PCOR, GPIO4_PDDR, GPIO4_PDOR, GPIO4_PSOR, GPIO4_PTOR};
use crate::nxp::syscon::SYSCON_AHBCLKCTRL0;
use crate::reg::{set_bits, write_reg};

pub const RED: u32 = 1 << 18;
pub const GREEN: u32 = 1 << 19;
pub const BLUE: u32 = 1 << 17;

pub const YELLOW: u32 = RED | GREEN;
pub const ALL: u32 = RED | GREEN | BLUE;

pub fn init() {
    set_bits(SYSCON_AHBCLKCTRL0, (1 << 17) | (1 << 23));

    set_bits(GPIO4_PDOR, ALL);
    set_bits(GPIO4_PDDR, ALL);
}

pub fn on(color: u32) {
    write_reg(GPIO4_PCOR, color);
}

pub fn off(color: u32) {
    write_reg(GPIO4_PSOR, color);
}
pub fn toggle(color: u32) {
    write_reg(GPIO4_PTOR, color);
}

pub fn set(color: u32) {
    off(ALL & !color);
    on(color);
}