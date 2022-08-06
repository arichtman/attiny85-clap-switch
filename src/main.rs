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
// S&mut o might need to use LEDs to signal a calibration period on boot?
// would still prefer hard-coded sample buffer for constistency and persistence
// TODO: Create reference pattern
static REFERENCE_PATTERN: u128 = 123456789;
// This is 25% error rate, pretty high but testing will reveal if that's the case
// TODO: Tune error rate
static DIFFERENCE_THRESHOLD: u8 = 32;
// TODO: Tune sound threshold
static SOUND_THRESHOLD: u8 = 128;

// have to use global/static object as ISR cannot receive arguments
// there _may_ be a way to `steal()` or `borrow()` the pins
// but this seems to be acceptable given we enforce boundaries
// on pin access
static mut PERIPHERALS: Option<hal::Peripherals> = None;

#[hal::entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();
    /* need to find out why this is different
    let pins = hal::pins!(peripherals);
    let mut pb1 = pins.pb1.borrow();
     */
    // need to set COM0A0, COM0A1, COM0B0 COM0B1, WGM00, WGM01 to 0 in TCCR0A
    // I'm unsure why the raw bit writing takes u8 when it looks like only 2 bits
    // and we're lacking convenience functions like we have for TCCR0B
    // whatever, setting 0 should wipe the bits across all of them
    let _ = peripherals
        .TC0
        .tccr0a
        .write(|w| w.wgm0().bits(0_u8).com0a().bits(0_u8).com0b().bits(0_u8));

    // need to set CS02, CS00 to 1, CS01, WGM02 to 0 in TCR0B
    // luckily we have a convenience function for the CS register
    let _ = peripherals
        .TC0
        .tccr0b
        .write(|w| w.cs0().prescale_1024().wgm02().clear_bit());

    //region ADC
    // https://www.gadgetronicx.com/attiny85-adc-tutorial-interrupts/
    // need to set REFS0, REFS1 to 0 in ADMUX
    // Select ADC0 on PB5 with MUX value 0000
    let _ = peripherals.ADC.admux.write(|w| {
        w
            // set Voltage Reference to Vcc (5v)
            .refs()
            .vcc()
            // Set ADC0/PB5 for conversion
            .mux()
            .adc0()
            // unsure of default so safer to explicitly set to right-shifted results
            // TODO: check spec sheet for default, remove if not required
            .adlar()
            .clear_bit()
    });
    let _ = peripherals.ADC.adcsra.write(|w| {
        w
            // enable ADC
            .aden()
            .set_bit()
            // enable auto triggering
            .adate()
            .set_bit()
            // set lowest clock frequency (8/16MHz / 128 is still __way__ faster than our ISR on overflow at 62 Hz)
            .adps()
            .prescaler_128()
    });
    // Looks like we can run an ADC conversion on TC0 overflow and that in turn can run it's own ISR
    // TODO: Look into moving buffer fill code into ADC ISR
    let _ = peripherals.ADC.adcsrb.write(
        |w| {
            w
                // set to free run mode
                .adts()
                .free()
        }, // set to read on timer 0 overflow
           // it's unclear if this will be a race condition and we may read stale data
           // in the actual ISR for overflow, perhaps if we check the ADC read bit
           // we can ensure we've the latest reading
           // .adts().tc0_ovf()
    );
    // let pins = hal::pins!(peripherals);
    // let adc = hal::Adc::new(peripherals.ADC, Default::default());
    // adc.read_blocking(pins)

    // unsafe use of mutable static
    // could we pass ownership of peripherals to the static?
    unsafe {
        PERIPHERALS = Option::Some(peripherals);
    }

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
    todo!();
    // bump everything left one spot, I'm unsure what will happen to the overflow but we don't really care anyway
    SOUND_HISTORY <<= 1;
    let peripherals = PERIPHERALS.as_ref().unwrap();
    let reader = peripherals.ADC.adc.read();
    reader.bits();
    let mic_value = peripherals.ADC.adc.read().bits();
    // TODO: Sort out pin control to read ADC
    // IIRC division requires 2 same types and yields same output type
    // This would mean our u8 defaults to like // or floor operator, which suits us
    // SOUND_HISTORY |= (mic_value / SOUND_THRESHOLD) as u128;
    // Calculate different bits
    let diff = SOUND_HISTORY ^ REFERENCE_PATTERN;
    if diff.count_ones() <= DIFFERENCE_THRESHOLD as u32 {
        // Use pin control to toggle relay
    }
}
