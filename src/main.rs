#![no_std]
#![no_main]

mod arm;
mod drivers;
mod nxp;
mod reg;
mod startup;

use core::panic::PanicInfo;

use crate::arm::systick::delay_ms;
use crate::drivers::led::{self, BLUE, GREEN, YELLOW};

#[panic_handler]
fn panic_handler(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

fn main() -> ! {
    arm::systick::init();
    led::init();
    led::on(BLUE);
    loop {
        led::set(GREEN);
        delay_ms(1000);
        led::set(BLUE);
        delay_ms(1000);
    }
}
