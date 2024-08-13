# attiny85-clap-switch

Project to build a sound-activated power toggle switch

[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white)](https://github.com/pre-commit/pre-commit)

## Process

0. Obtain components
0. Flash chip
0. (optional) Breadboard and debug
0. Put components together
0. Impress 80s kids and toddlers alike!

## Components

All required unless marked otherwise. AliExpress links for convenience, not an endorsement of site, sellers, or products.
I have no affiliation with any of the purchase links and receive no commission or consideration if you use them.

- [Digispark ATTiny85](https://www.aliexpress.com/item/32724114567.html) (could be another model but you're on your own)
- [Microphone](https://www.aliexpress.com/item/32639718325.html)
- [Power source](https://www.aliexpress.com/item/32845177402.html)
  (Optional, you could use the USB to power the ATTiny.
  However, if you want to embed it somewhere and you need mains power to your appliance anyway...)
- Micro USB data cable (for programming)
- [5kΩ Potentiometer](https://www.aliexpress.com/item/32783863247.html) (optional, for adjusting thresholds)
- [Breadboard kit](https://www.aliexpress.com/item/4000689310993.html) (optional, for prototyping and testing)
- Assembly bits including; soldering iron, heat shrink, solder, wires, header pins etc

## Design

If we're going to measure anything time-sensitive, we cannot rely on `noop` or loops.
Present thinking is to bind timer zero to internal clock @16.5Mhz mode with prescaling at maximum (1024).
This'll set timer to 16,500,00 / 1024 = 16,113. That's frequency 1/16113 = 0.000,062 so a count every 62 microseconds.
Max count before overflow is 256 so interrupt timing will be 256 * 0.000062 = 0.015872 which is about every 16 milliseconds.
When the ISR trips we'll read the ADC for the mic value and store it to a FIFO queue in working memory.
If the read is low and we're not in a listen state, that means no initial event to listen, and can simply not write to queue.
When the initial read triggers high, we can start the listen sequence, maybe via flag.
While the flag is active we'll count to a number of reads, enough to get about 2 seconds of audio data.
At 16 milliseconds between reads that would be 2000 / 16 = 125, we'll round that to 128 for neatness.
Available working memory is 512 bytes so we could store quite the queue of readings @ 256-bit granularity.
I was thinking about looking for features like 2 local maximums in the first 50% of data points and one maximum in the last 25% but we're getting into math-heavy territory,
I'm not sure the chip will have space nor time to do calculus, nor that it would even be advantageous, it also will hog a bunch more memory.
For these reasons, we'll proess the readings on inpet into boolean/binary bits, indicating a "loud" or "quiet" read, and push that to the queue.
Setting thresholds for loud and quiet reads might be tricky.
It may be possible to maintain a rolling average from a secondary queue of readings to determine background noise, but that's more complication, and can go in the enhancement bin.
I think we'll hook up more potentiometers to allow dynamic adjustment for development and likely hard-code the values.
Maximum control would be to set separate thresholds for high and low but this clashes with reducing the values to bit-level granularity.
Perhaps we'll include a control for this when installed, undecided.
When ISR runs and listen count is 128 we'll do a bitwise xor between a reference queue in EEPROM as a CONST.
This'll yield only bits different, which we can then aggregate by SUM.
This differential can be divided by 128 to yield a loose % difference between what was recorded and what's expected.
It doesn't account for timing very well but should be super fast and is much easier to tune with only one control mechanism, threshold for % difference.
This control can be set via potentiometer for dynamic adjustment, setting to 100% similarity would effectively disable the mechanism.
The threshold should have a higher floor value though, as setting it too low would result in just about anything triggering it.
We can achieve this by scaling the potentiometer value to 100 - $Floor.

## Development setup

### Magic way

Use nix-direnv and Nix

### PITA

#### Prerequisites

**Note:** This method is not completely tested, please submit issues/feedback or just figure it out

Required:

- rustup
- gcc-avr
- avr-libc
- pkg-config
- libudev-dev
- Patience

Optional (but recommended):

- Python 3.9 or later
- Poetry

```Bash
# Documentation indicates we require 1.63.0 or later
rustc --version
# Update
rustup toolchain install nightly
# Confirm version and use
rustc --version
rustup override set nightly

# If using pre-commit, install and use environment
poetry install
poetry shell
pre-commit install --install-hooks
# Verify working
pre-commit run --all
```

## Compiling

```Bash
cargo build -Z build-std=core --release
```

## Flashing

Convert file format and move to host machine file system.

```bash
# Convert to hex file
# Need to find out if -S to strip is desirable or does nothing
#   kinda moot until we hit the 8k EEPROM limit anyhow
objcopy -O ihex target/avr-attiny85/release/attiny85-clap-switch.elf target/avr-attiny85/release/attiny85-clap-switch.hex
# (optional) copy to conveniently accessible Windows location
cp target/avr-attiny85/release/attiny85-clap-switch.hex /mnt/d/
```

### Windows

Download and extract the latest [micronucleus release](https://github.com/micronucleus/micronucleus/releases).
You can use my [Shovel bucket](https://github.com/arichtman/shovel-bucket) if you prefer a managed install option.

```powershell
micronucleus.exe D:\attiny85-clap-switch.hex
# Now plug the board in and it'll flash
```

### Troubleshooting

Error: `can't find crate for test`
Fix: Set `rust-analyzer.checkOnSave.allTargets = false`

Error: `proc macro ``entry`` not expanded: failed to write request: Broken pipe (os error 32)rust-analyzerunresolved-proc-macro`
Fix: Disable proc macros? Upgrade to nightly build of rust-analyzer?

Error: `language item required, but not found: eh_personality`
Fix: Ensure target is set in `.cargo/config.toml` and disable `--all-targets`

## References

- Eb, for indespensible advice
- [Setup guide for flashing](https://www.best-microcontroller-projects.com/digispark-attiny85-arduino-install.html)
- [Pinout and spec guide](https://www.etechnophiles.com/attiny85-pinout-specs-guide/)
- [Rust reference](https://book.avr-rust.com/)
- [Emulator](https://wokwi.com/)
- [Sample repo](https://github.com/q231950/avr-attiny85-rust)
- [Some build troubleshooting](https://nercury.github.io/rust/embedded/experiments/2018/04/29/rust-embedded-01-discovery-vl-flipping-bits.html)
- [Interrupt tutorial](https://www.gadgetronicx.com/attiny85-timer-tutorial-generating-time-delay-interrupts/)
