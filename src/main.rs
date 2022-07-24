#![no_std]
#![no_main]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

#[cfg(feature = "rt")]
use hal::entry;

#[entry]
fn main() -> ! {
    loop {}
}
