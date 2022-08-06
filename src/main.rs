#![no_std]
#![no_main]
// enables interrupt feature?
#![feature(abi_avr_interrupt)]
// to allow static mut peripherals
// #![feature(const_option)]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

use avr_device::interrupt;

// Unsure how the compiler will treat this.
// I don't believe there's a single bit primitve so _maybe_ bool will be same?
// static mut BIT_BUFFER: [bool; 128] = [false; 128];
// given we'll be doing bitwise operations we might as well start that way
// initialise a 128-bit piece of memory to all 0s
static mut SOUND_HISTORY: u128 = 0_u128;
// Need to figure out a good way to create sample buffers from real audio
// Don't think I have a way to retrieve data from running chip
// So might need to use LEDs to signal a calibration period on boot?
// would still prefer hard-coded sample buffer for constistency and persistence
// TODO: Create reference pattern
static REFERENCE_PATTERN: u128 = 123456789;
// This is 25% error rate, pretty high but testing will reveal if that's the case
// TODO: Tune error rate
static DIFFERENCE_THRESHOLD: u8 = 32;
// TODO: Tune sound threshold
static SOUND_THRESHOLD: u8 = 128;

// static mut PERIPHERALS: hal::Peripherals = _;

// static mut PERIPHERALS: Option<hal::Peripherals> = None;
pub struct PinControl {
    periph: Option<hal::Peripherals>,
}

static mut PIN_CONTROL: PinControl = PinControl { periph: None };

#[hal::entry]
unsafe fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();
    /* need to find out why this is different
    let pins = hal::pins!(peripherals);
    let mut pb1 = pins.pb1.borrow();
     */
    // need to set COM0A0, COM0A1, COM0B0 COM0B1, WGM00, WGM01 to 0 in TCCR0A
    // I'm unsure why the raw bit writing takes u8 when it looks like only 2 bits
    // and we're lacking convenience functions like we have for TCCR0B
    // whatever, setting 0 should wipe the bits across all of them
    let _ = &peripherals
        .TC0
        .tccr0a
        .write(|w| w.wgm0().bits(0_u8).com0a().bits(0_u8).com0b().bits(0_u8));

    // need to set CS02, CS00 to 1, CS01, WGM02 to 0 in TCR0B
    // luckily we have a convenience function for the CS register
    let _ = &peripherals
        .TC0
        .tccr0b
        .write(|w| w.cs0().prescale_1024().wgm02().clear_bit());
    PIN_CONTROL.periph = Option::Some(peripherals);

    loop {
        // could put a noop here but it makes more sense to just let main run to completion
        // I just can't solve this error yet
        // implicitly returns `()` as its body has no tail or `return` expression
    }
}

// https://github.com/Rahix/avr-hal/blob/main/examples/arduino-uno/src/bin/uno-timer.rs
// https://www.tutorialspoint.com/rust/rust_bitwise_operators.htm
#[interrupt(attiny85)]
unsafe fn TIMER0_OVF() {
    // bump everything left one spot, I'm unsure what will happen to the overflow but we don't really care anyway
    SOUND_HISTORY <<= 1;
    let peripherals = &PIN_CONTROL.periph;
    peripherals.as_ref();
    let mic_value = 128_u8;
    // TODO: Sort out pin control to read ADC
    // IIRC division requires 2 same types and yields same output type
    // This would mean our u8 defaults to like // or floor operator, which suits us
    SOUND_HISTORY |= (mic_value / SOUND_THRESHOLD) as u128;
    // Calculate different bits
    let diff = SOUND_HISTORY ^ REFERENCE_PATTERN;
    if diff.count_ones() <= DIFFERENCE_THRESHOLD as u32 {
        // Use pin control to toggle relay
    }
}
