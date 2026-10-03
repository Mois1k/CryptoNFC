#![no_std]
#![no_main]

mod gpio;
mod reg;
mod scb;
mod startup;
mod syscon;
mod systick;

use core::panic::PanicInfo;

use crate::gpio::{GPIO4_PDDR, GPIO4_PDOR, GPIO4_PTOR};
use crate::reg::{set_bits, write_reg};
use crate::syscon::SYSCON_AHBCLKCTRL0;
use crate::systick::delay_ms;

#[panic_handler]
fn panic_handler(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

fn main() -> ! {
    set_bits(SYSCON_AHBCLKCTRL0, (1 << 17) | (1 << 23));

    set_bits(GPIO4_PDOR, 1 << 18);
    set_bits(GPIO4_PDDR, 1 << 18);

    systick::init();
    loop {
        write_reg(GPIO4_PTOR, 1 << 18);
        delay_ms(1000);
    }
}
