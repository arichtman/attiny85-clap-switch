# attiny85-clap-switch

Project to build a sound-activated power toggle switch

[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white)](https://github.com/pre-commit/pre-commit)

## Process

0. Obtain components
0. Flash chip
0. (optional) Breadboard and debug
0. Put components together
0. Impress 80s children and toddler alike!

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

## References

- Eb, for indespensible advice
- [Setup guide for flashing](https://www.best-microcontroller-projects.com/digispark-attiny85-arduino-install.html)
- [Pinout and spec guide](https://www.etechnophiles.com/attiny85-pinout-specs-guide/)
- [Rust reference](https://book.avr-rust.com/)
- [Emulator](https://wokwi.com/)
- [Sample repo](https://github.com/q231950/avr-attiny85-rust)
- [Some build troubleshooting](https://nercury.github.io/rust/embedded/experiments/2018/04/29/rust-embedded-01-discovery-vl-flipping-bits.html)

## Development setup

### Magic way

0. Open project using VSCode Devcontainers

NB: You will need the mount directories to at least exist for this to load correctly.
`mkdir -p $HOME/.local/share/cargo/registry $HOME/.config`

### PITA

#### Prerequisites

**Note:** This method is not completely tested, please submit issues/feedback or just figure it out

Required:

- rustup
- gcc-avr
- avr-libc
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
