// required for AVR
#![no_std]
#![no_main]
// enables interrupt functions
#![feature(abi_avr_interrupt)]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

use embedded_hal::blocking::delay::DelayMs;
use hal::{clock::MHz16, delay::Delay, port::mode::*, port::Pin, port::*, Adc, Peripherals};

// have to use global/static object as ISR cannot receive arguments, there _may_ be a way to `steal()` or `borrow()` the pins
// but this seems to be a valid approach. Wish I could dispose of the main function's handle on it though for surety
// Wrapping and unwrapping is also a bit rudimentary
// TODO: Consult with someone to determine if the Option route is appropriate
// TODO: Look into genericising this so it's not coupled to the particular pin we're using

#[hal::entry]
/// This file is presently configured for testing/prototyping
/// Current aim is to check that ADC in free run read mode + Timer0 interrupts are running
/// by adjusting the blink delay depending on the ADC read value. We can use the mic with
/// a constant volume input or a potentiometer for this purpose. I would like to determine mic
/// sensitivity and whether we need to enable gain on the ADC for it.
fn main() -> ! {
    // take() only works one time, so we'll need to pull out the bits we need and assign them to global/static variables for use in ISRs
    let device_peripherals = hal::pac::Peripherals::take().unwrap();

    initialize_adc(&device_peripherals);
    // get mutable ADC
    let mut device_adc: Adc<MHz16> = hal::Adc::new(device_peripherals.ADC, Default::default());
    let device_pins = hal::pins!(device_peripherals);
    // TODO: look into doing a partial or something so the adc reference is already passed
    // https://docs.rs/partial_application/latest/partial_application/
    // Might need to set the static reference to our instance and use that
    // else our instance would drop out of scope and the partial fail
    // I played with this and because of the Option wrapping it's not so straightforward to
    // assign ADC = adc_device and then use that when calling into_analog_input()
    let mut led_output = device_pins.pb0.into_output();

    let analog_input = device_pins.pb4.into_analog_input(&mut device_adc);

    let mut delay = Delay::<MHz16>::new();
    loop {
        let analog_value = analog_input.analog_read(&mut device_adc);
        let delay_time = match analog_value {
            0..=20 => 512_u16,
            21..=127 => 768_u16,
            128..=255 => 1024,
            256..=511 => 1280,
            512..=767 => 1536,
            768..=1023 => 2048,
            _ => 2560,
        };
        delay.delay_ms(delay_time);
        led_output.toggle();
    }
}

// TODO: find out if better to store constants perpetually in working memory by using static
// or in the function itself as just a const. I bet the compiler just hard-codes the values anyhow...

// https://www.gadgetronicx.com/attiny85-adc-tutorial-interrupts/
fn initialize_adc(peripherals: &Peripherals) {
    // need to set REFS0, REFS1 to 0 in ADMUX
    // Select ADC2 on PB4 with MUX value 0b0010
    let _ = peripherals.ADC.admux.write(|w| {
        w
            // set Voltage Reference to Vcc (5v)
            .refs()
            .vcc()
            // Set ADC2/PB4 for conversion
            .mux()
            .adc2()
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
    let _ = peripherals.ADC.adcsrb.write(|w| {
        w
            // set to free run mode
            .adts()
            .free()
    });
}
