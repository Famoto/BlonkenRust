use anyhow::Result;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{Gpio10, InputOutput, PinDriver, Pull};

use crate::digitus16::DIGITS;

pub const TEMPERATURE_GPIO: u8 = 10;

const CONVERT_T: u8 = 0x44;
const READ_SCRATCHPAD: u8 = 0xbe;
const SKIP_ROM: u8 = 0xcc;
const SCRATCHPAD_LEN: usize = 9;

// 12-bit DS18B20 conversions take up to 750 ms. With a 10 ms app tick this
// leaves a little slack without stalling the display loop.
const CONVERSION_TICKS: u16 = 80;
const RETRY_TICKS: u16 = 100;

type OneWirePin = PinDriver<'static, InputOutput>;

pub struct TemperatureSensor {
    bus: OneWireBus,
    ticks_until_read: u16,
    last_tenths_celsius: Option<i16>,
    sensor_present: bool,
}

impl TemperatureSensor {
    pub fn new(pin: Gpio10<'static>) -> Result<Self> {
        let mut sensor = Self {
            bus: OneWireBus::new(pin)?,
            ticks_until_read: 0,
            last_tenths_celsius: None,
            sensor_present: false,
        };

        sensor.start_conversion()?;

        Ok(sensor)
    }

    pub fn poll(&mut self) -> Result<()> {
        if self.ticks_until_read > 0 {
            self.ticks_until_read -= 1;
            return Ok(());
        }

        if self.sensor_present {
            if let Some(tenths_celsius) = self.read_temperature()? {
                self.last_tenths_celsius = Some(tenths_celsius);
            } else {
                self.sensor_present = false;
            }
        }

        self.start_conversion()
    }

    pub fn display_text(&self) -> [u8; DIGITS] {
        match self.last_tenths_celsius {
            Some(tenths_celsius) => temperature_text(tenths_celsius),
            None => *b"NO TMP",
        }
    }

    fn start_conversion(&mut self) -> Result<()> {
        if !self.bus.reset()? {
            self.ticks_until_read = RETRY_TICKS;
            return Ok(());
        }

        self.sensor_present = true;
        self.bus.write_byte(SKIP_ROM)?;
        self.bus.write_byte(CONVERT_T)?;
        self.ticks_until_read = CONVERSION_TICKS;

        Ok(())
    }

    fn read_temperature(&mut self) -> Result<Option<i16>> {
        if !self.bus.reset()? {
            return Ok(None);
        }

        self.bus.write_byte(SKIP_ROM)?;
        self.bus.write_byte(READ_SCRATCHPAD)?;

        let mut scratchpad = [0u8; SCRATCHPAD_LEN];
        for byte in &mut scratchpad {
            *byte = self.bus.read_byte()?;
        }

        if crc8(&scratchpad) != 0 {
            return Ok(None);
        }

        let raw = i16::from_le_bytes([scratchpad[0], scratchpad[1]]);
        Ok(Some(((i32::from(raw) * 10) / 16) as i16))
    }
}

struct OneWireBus {
    pin: OneWirePin,
}

impl OneWireBus {
    fn new(pin: Gpio10<'static>) -> Result<Self> {
        let mut pin = PinDriver::input_output_od(pin, Pull::Up)?;
        pin.set_high()?;

        Ok(Self { pin })
    }

    fn reset(&mut self) -> Result<bool> {
        self.pin.set_low()?;
        Ets::delay_us(480);
        self.pin.set_high()?;
        Ets::delay_us(70);
        let present = self.pin.is_low();
        Ets::delay_us(410);

        Ok(present)
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        for bit in 0..8 {
            self.write_bit(byte & (1 << bit) != 0)?;
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut byte = 0u8;

        for bit in 0..8 {
            if self.read_bit()? {
                byte |= 1 << bit;
            }
        }

        Ok(byte)
    }

    fn write_bit(&mut self, high: bool) -> Result<()> {
        self.pin.set_low()?;

        if high {
            Ets::delay_us(6);
            self.pin.set_high()?;
            Ets::delay_us(64);
        } else {
            Ets::delay_us(60);
            self.pin.set_high()?;
            Ets::delay_us(10);
        }

        Ok(())
    }

    fn read_bit(&mut self) -> Result<bool> {
        self.pin.set_low()?;
        Ets::delay_us(6);
        self.pin.set_high()?;
        Ets::delay_us(9);
        let high = self.pin.is_high();
        Ets::delay_us(55);

        Ok(high)
    }
}

fn temperature_text(tenths_celsius: i16) -> [u8; DIGITS] {
    let rounded = if tenths_celsius >= 0 {
        (tenths_celsius + 5) / 10
    } else {
        (tenths_celsius - 5) / 10
    };
    let mut text = *b"    C ";
    let negative = rounded < 0;
    let abs_value = i32::from(rounded).abs();

    if abs_value > 999 || (negative && abs_value > 99) {
        return if negative { *b"COLD  " } else { *b" HOT  " };
    }

    text[2] = digit(abs_value % 10);

    if abs_value >= 10 {
        text[1] = digit((abs_value / 10) % 10);
    }

    if abs_value >= 100 {
        text[0] = digit((abs_value / 100) % 10);
    } else if negative {
        text[if abs_value >= 10 { 0 } else { 1 }] = b'-';
    }

    text
}

fn digit(value: i32) -> u8 {
    b'0' + value as u8
}

fn crc8(bytes: &[u8]) -> u8 {
    let mut crc = 0u8;

    for byte in bytes {
        let mut value = *byte;

        for _ in 0..8 {
            let mix = (crc ^ value) & 0x01;
            crc >>= 1;

            if mix != 0 {
                crc ^= 0x8c;
            }

            value >>= 1;
        }
    }

    crc
}
