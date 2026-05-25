# BlonkenRust

Rust firmware for the ESP32-C3-based [blonkenclick](https://github.com/wenzellabs/blonkenclick)
clock kit. It drives the six-digit RGB 16-segment display over SPI, renders animated
HSB-colored text, and adjusts display intensity from the onboard light sensor.

This is an independent firmware experiment built for the blonkenclick hardware.
Credit for the hardware design and original project goes to
[Wenzel Labs / blonkenclick](https://github.com/wenzellabs/blonkenclick).

## Build

```bash
cargo check --locked
```

With an ESP32-C3 connected and `espflash` installed:

```bash
cargo run --release
```
