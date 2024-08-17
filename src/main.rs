// required for AVR
#![no_std]
#![no_main]
// enables interrupt functions
#![feature(abi_avr_interrupt)]

extern crate attiny_hal as hal;
extern crate avr_device;
extern crate panic_halt;

pub use avr_device::interrupt;
use core::mem;
use embedded_hal::blocking::delay::DelayMs;
use hal::{clock::MHz16, delay::Delay, port::mode::*, port::Pin, port::*, Adc, Peripherals};

// have to use global/static object as ISR cannot receive arguments, there _may_ be a way to `steal()` or `borrow()` the pins
//   but this seems to be a valid approach. Wish I could dispose of the main function's handle on it though for surety
// TODO: Consult with someone to determine if this is an appropriate use of statics
// TODO: Look into genericising this so it's not coupled to the particular pin we're using
static mut LED_OUTPUT: mem::MaybeUninit<Pin<Output, PB0>> = mem::MaybeUninit::uninit();
static mut ADC_INPUT: mem::MaybeUninit<Pin<Analog, PB3>> = mem::MaybeUninit::uninit();
static mut DEVICE_ADC: mem::MaybeUninit<Adc<MHz16>> = mem::MaybeUninit::uninit();

static mut DELAY_TIME: u16 = 512_u16;

// #[hal::entry]
#[no_mangle]
/// This file is presently configured for testing/prototyping
/// Current aim is to check test the TC0 ISR reading the free-read ADC
fn main() -> ! {
    // take() only works one time, so we'll need to pull out the bits we need and assign them to global/static variables for sharing with ISR(s)
    let device_peripherals = hal::pac::Peripherals::take().unwrap();
    // I'm blocked from moving these later as a couple functions in the unsafe block do a partial borrow against device_peripherals.
    //   I would really prefer to initialise the interrupts AFTER initialising the global variables, I worry that an ISR tripping early
    //   could lead to use of an uninitialised i/o object. At least with ADC in free read mode that's OK
    // To try and guard the TC0 ISR I'll globally disable interrupts, then enable explicitly after globals are initialised
    avr_device::interrupt::disable();

    initialize_adc(&device_peripherals);
    initialize_timer(&device_peripherals);
    // not loving how much of this is unsafe code
    unsafe {
        DEVICE_ADC =
            mem::MaybeUninit::new(hal::Adc::new(device_peripherals.ADC, Default::default()));
    }
    let device_pins = hal::pins!(device_peripherals);
    unsafe {
        // Initialise our shared object
        LED_OUTPUT = mem::MaybeUninit::new(device_pins.pb0.into_output());
        ADC_INPUT = mem::MaybeUninit::new(
            device_pins
                .pb3
                .into_analog_input(DEVICE_ADC.assume_init_mut()),
        );
        // something something block compiler reorders that might put this initialisation
        //   AFTER we attempt to use the variable
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    unsafe {
        // now all globals are initialised we can enable interrupts
        avr_device::interrupt::enable();
    }

    let mut delay = Delay::<MHz16>::new();
    loop {
        unsafe {
            delay.delay_ms(DELAY_TIME);
            LED_OUTPUT.assume_init_mut().toggle();
        }
    }
}

#[avr_device::interrupt(attiny85)]
unsafe fn TIMER0_OVF() {
    // TODO: look into doing a partial or something so the adc reference is already passed
    // https://docs.rs/partial_application/latest/partial_application/
    // Might need to set the static reference to our device_adc instance and use that lest it drop out of scope

    DELAY_TIME = match ADC_INPUT
        .assume_init_mut()
        .analog_read(DEVICE_ADC.assume_init_mut())
    {
        0..=20 => 512,
        21..=127 => 768,
        128..=255 => 1024,
        256..=511 => 1280,
        512..=767 => 1536,
        768..=1023 => 2048,
        _ => panic!(),
    };
}

// https://www.gadgetronicx.com/attiny85-adc-tutorial-interrupts/
fn initialize_adc(peripherals: &Peripherals) {
    // need to set REFS0, REFS1 to 0 in ADMUX
    // Select ADC2 on PB4 with MUX value 0b0010
    peripherals.ADC.admux.write(|w| {
        w
            // set Voltage Reference to Vcc (5v)
            .refs()
            .vcc()
            // Set ADC2/PB4 for conversion
            .mux()
            .adc3()
            // unsure of default so safer to explicitly set to right-shifted results
            // TODO: check spec sheet for default, remove if not required
            .adlar()
            .clear_bit()
    });
    peripherals.ADC.adcsra.write(|w| {
        w
            // enable ADC
            .aden()
            .set_bit()
            // enable auto triggering
            .adate()
            .set_bit()
            // set lowest clock frequency (8/16MHz / 128 is still __way__ faster than our ISR on overflow at ~62.5 Hz)
            .adps()
            .prescaler_128()
    });
    // Looks like we can run an ADC conversion on TC0 overflow and that in turn can run it's own ISR
    // TODO: Look into moving buffer fill code into ADC ISR
    peripherals.ADC.adcsrb.write(|w| {
        w
            // set to free read
            .adts()
            .free()
    });
}

fn initialize_timer(peripherals: &Peripherals) {
    // need to set COM0A0, COM0A1, COM0B0 COM0B1, WGM00, WGM01 to 0 in TCCR0A
    // I'm unsure why the raw bit writing takes u8 when it looks like only 2 bits
    // and we're lacking convenience functions like we have for TCCR0B
    // whatever, setting 0 should wipe the bits across all of them
    peripherals
        .TC0
        .tccr0a
        .write(|w| w.wgm0().bits(0_u8).com0a().bits(0_u8).com0b().bits(0_u8));

    // need to set CS02, CS00 to 1, CS01, WGM02 to 0 in TCR0B
    // luckily we have a convenience function for the CS register
    peripherals
        .TC0
        .tccr0b
        .write(|w| w.cs0().prescale_1024().wgm02().clear_bit());
}
