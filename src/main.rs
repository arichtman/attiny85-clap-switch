#![no_std]
#![no_main]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

use hal::clock::*;
use hal::delay::*;

use embedded_hal::blocking::delay::DelayMs;

#[hal::entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();
    let mut delay = Delay::<MHz1>::new();
    let ports = &peripherals.PORTB.pinb;
    loop {
        ports.write(|w| w.pb1().set_bit());
        // these delays are far shorter than 1000 milliseconds
        // I wonder if it's clocked at 16mhz out-the-box by digispark...
        delay.delay_ms(1000_u16);
        ports.write(|w| w.pb1().clear_bit());
        delay.delay_ms(1000_u16);
    }
}
