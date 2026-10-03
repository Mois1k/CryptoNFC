#![no_std]
#![no_main]

mod gpio;
mod reg;
mod scb;
mod startup;
mod syscon;

use core::{arch::asm, panic::PanicInfo};

use crate::gpio::{GPIO4_PDDR, GPIO4_PDOR, GPIO4_PTOR};
use crate::reg::{set_bits, write_reg};
use crate::syscon::SYSCON_AHBCLKCTRL0;

#[panic_handler]
fn panic_handler(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

fn main() -> ! {
    set_bits(SYSCON_AHBCLKCTRL0, (1 << 17) | (1 << 23));

    set_bits(GPIO4_PDOR, 1 << 18);
    set_bits(GPIO4_PDDR, 1 << 18);

    loop {
        write_reg(GPIO4_PTOR, 1 << 18);
        for _ in 0..12000000 {
            unsafe { asm!("nop") };
        }
    }
}
