#![no_std]
#![no_main]

mod arm;
mod drivers;
mod nxp;
mod reg;
mod startup;

use core::panic::PanicInfo;

use crate::arm::systick::delay_ms;
use crate::drivers::led::YELLOW;
use crate::nxp::gpio::{GPIO4_PDDR, GPIO4_PDOR, GPIO4_PTOR};
use crate::nxp::syscon::SYSCON_AHBCLKCTRL0;
use crate::reg::{set_bits, write_reg};

#[panic_handler]
fn panic_handler(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

fn main() -> ! {
    set_bits(SYSCON_AHBCLKCTRL0, (1 << 17) | (1 << 23));

    set_bits(GPIO4_PDOR, YELLOW);
    set_bits(GPIO4_PDDR, YELLOW);

    arm::systick::init();
    loop {
        write_reg(GPIO4_PTOR, YELLOW);
        delay_ms(1000);
    }
}
