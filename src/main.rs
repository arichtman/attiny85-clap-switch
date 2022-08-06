// required for AVR
#![no_std]
#![no_main]
// enables interrupt functions
#![feature(abi_avr_interrupt)]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

use avr_device::interrupt;
use hal::port::Pin;
use hal::port::PB5;
use hal::Peripherals;
use hal::{clock::MHz16, port::mode::Analog};

// We'll be doing bitwise operations for speed and simplicity so we'll use this in place of an array
// initialise a 128-bit piece of memory to all 0s
static mut SOUND_HISTORY: u128 = 0_u128;

// Need to figure out a good way to create sample buffers from real audio
// Don't think I have a way to retrieve data from running chip
// We may use LEDs to signal a calibration period on boot for development
// but would still prefer hard-coded sample buffer for consistency and persistence
// TODO: Create reference pattern
static REFERENCE_PATTERN: u128 = 123456789;

// This is 25% error rate, pretty high but testing will reveal if that's the case
// TODO: Tune error rate
static DIFFERENCE_THRESHOLD: u8 = 32;

// TODO: Tune sound threshold
static SOUND_THRESHOLD: u16 = 128;

// Vin is scaled * 1024 / Vref (in our case, 5)
// 1024 is consistent with the 10 bits stored in ADCL/H
static MAX_ADC_VALUE: u16 = 1024;

// have to use global/static object as ISR cannot receive arguments, there _may_ be a way to `steal()` or `borrow()` the pins
// but this seems to be a valid approach. Wish I could dispose of the main function's handle on it though for surety
// Wrapping and unwrapping is also a bit rudimentary
// TODO: Consult with someone to determine if the Option route is appropriate
// TODO: Look into genericising this so it's not coupled to the particular pin we're using
static mut ANALOG_INPUT_PB5: Option<Pin<Analog, PB5>> = None;
static mut ADC: Option<hal::Adc<MHz16>> = None;

#[hal::entry]
fn main() -> ! {
    // take() only works one time, so we'll need to pull out the bits we need and assign them to global/static variables for use in ISRs
    let device_peripherals = hal::pac::Peripherals::take().unwrap();

    initialize_timer_0_1024(&device_peripherals);
    initialize_adc_free_pb5(&device_peripherals);
    // partial borrow here needs to be out of scope of peripherals use in initialization
    let device_pins = hal::pins!(device_peripherals);

    let mut device_adc: hal::Adc<MHz16> = hal::Adc::new(device_peripherals.ADC, Default::default());
    // TODO: look into doing a partial or something so the adc reference is already passed
    // https://docs.rs/partial_application/latest/partial_application/
    // Might need to set the static reference to our instance and use that
    // else our instance would drop out of scope and the partial fail
    // I played with this and because of the Option wrapping it's not so straightforward to
    // assign ADC = adc_device and then use that when calling into_analog_input()
    let analog_input_pb5 = device_pins.pb5.into_analog_input(&mut device_adc);

    // unsafe use of mutable static
    // could we pass ownership of peripherals to the static?
    unsafe {
        // PERIPHERALS = Option::Some(peripherals);
        ANALOG_INPUT_PB5 = Option::Some(analog_input_pb5);
        ADC = Some(device_adc);
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
    // bump everything left one spot, I'm unsure what will happen to the overflow but we don't really care anyway
    SOUND_HISTORY <<= 1;
    // TODO: Add check for ADC ready flag so we don't try to read dirty values
    // TODO: Clarify use of as_ref() and as_mut()
    let pin_reader = ANALOG_INPUT_PB5.as_ref().unwrap();
    let mut adc = ADC.as_mut().unwrap();
    let mic_value = pin_reader.analog_read(&mut adc);
    // IIRC division requires 2 same types and yields same output type
    // This would mean our u8 defaults to like // or floor operator, which suits us
    SOUND_HISTORY |= (mic_value / SOUND_THRESHOLD) as u128;
    // Calculate different bits
    let diff = SOUND_HISTORY ^ REFERENCE_PATTERN;
    if diff.count_ones() <= DIFFERENCE_THRESHOLD as u32 {
        // Use pin control to toggle relay
    }
}

fn initialize_adc_free_pb5(peripherals: &Peripherals) {
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
}

fn initialize_timer_0_1024(peripherals: &Peripherals) {
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
}
