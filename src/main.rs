#![no_std]
#![no_main]
// not sure, got it from https://doc.rust-lang.org/stable/embedded-book/start/exceptions.html
#![deny(unsafe_code)]
// enables interrupt feature?
#![feature(abi_avr_interrupt)]
// to allow static mut peripherals
// #![feature(const_option)]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

// use hal::Peripherals;
use hal::clock::*;
use hal::delay::*;

use embedded_hal::blocking::delay::DelayMs;

use avr_device::interrupt;

use core::borrow::Borrow;

#[hal::entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();

    // need to set COM0A0, COM0A1, COM0B0 COM0B1, WGM00, WGM01 to 0 in TCCR0A
    // I'm unsure why the raw bit writing takes u8 when it looks like only 2 bits
    // and we're lacking convenience functions like we have for TCCR0B
    // whatever, setting 0 should wipe the bits across all of them
    let _tcc_r0a = &peripherals
        .TC0
        .tccr0a
        .write(|w| w.wgm0().bits(0_u8).com0a().bits(0_u8).com0b().bits(0_u8));

    // need to set CS02, CS00 to 1, CS01, WGM02 to 0 in TCR0B
    // luckily we have a convenience function for the CS register
    let _tcc_r0b = &peripherals
        .TC0
        .tccr0b
        .write(|w| w.cs0().prescale_1024().wgm02().clear_bit());

    // TODO: work out if we need this from embedded_hal or we can do away with that crate
    let mut delay = Delay::<MHz16>::new();

    let ports = &peripherals.PORTB.pinb;
    // imma be real with you, idk about this borrowing stuff
    let tifr = &peripherals.TC0.tifr.borrow();

    let mut delay_time = 5000_u16;
    loop {
        // let mut pins = hal::Pins::new(portb);
        ports.write(|w| w.pb1().bit(!ports.read().pb1().bit()));
        delay.delay_ms(delay_time);
        // we'll test the interrupt is running by checking if the overflow triggers
        // need to check TOV0 in TIFR for manual overflow handling
        if tifr.read().tov0().bit_is_set() {
            delay_time = 1000_u16;
            tifr.reset();
        }
    }
}

/*
#[interrupt(attiny85)]
fn TIMER0_OVF() {
    // hal::pac::Peripherals::take() is a one-time operation, not sure how to borrow it out
    // but I can't call it only in here as I need to set the timer pins once,
    // I can't flag and set them in the interrupt as the interrupt would never trigger
    // static mut peripherals: Peripherals = hal::pac::Peripherals::take().unwrap();


    // https://doc.rust-lang.org/stable/embedded-book/start/interrupts.html
    // that says it's fine but rust analyzer is complaining
    // but it compiles fine??
    static mut COUNT: u8 = 0;
    *COUNT += 1;
    // 128 * 0.016ms = 2.04 seconds
    if *COUNT == 128 {
        let peripherals = hal::pac::Peripherals::take().unwrap();
        let ports = &peripherals.PORTB.pinb;
        ports.write(|w| w
            .pb1().bit(! ports.read().pb1().bit())
        );
        *COUNT = 0_u8;
    }
}
*/
