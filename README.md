# BlonkenRust

Rust firmware for the ESP32-C3-based [blonkenclick](https://github.com/wenzellabs/blonkenclick)
clock kit. It drives the six-digit RGB 16-segment display over SPI, renders animated
HSB-colored text, and adjusts display intensity from the onboard light sensor. A
button on an unused GPIO switches between the message and the ambient temperature
reading.

This is an independent firmware experiment built for the blonkenclick hardware.
Credit for the hardware design and original project goes to
[Wenzel Labs / blonkenclick](https://github.com/wenzellabs/blonkenclick).

## Hardware

- Display: `GPIO4` SCK, `GPIO6` MOSI
- Light sensor: `GPIO0`
- Ambient temperature sensor: `GPIO10` DS18B20/1-Wire line
- Mode button: `GPIO7` to GND, using the ESP32-C3 internal pull-up

## Build

```bash
cargo check --locked
```

With an ESP32-C3 connected and `espflash` installed:

```bash
cargo run --release
```
