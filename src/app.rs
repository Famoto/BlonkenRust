use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;

use crate::digitus16::{DIGITS, Digitus16, SPI_CLOCK_GPIO, SPI_DATA_GPIO};
use crate::light_sensor::{LIGHT_SENSOR_GPIO, LightSensor};

pub const FRAME_DELAY_MS: u32 = 10;

const TEXT: &[u8; DIGITS] = b"FCKAFD";
const STATUS_INTERVAL_TICKS: u16 = 100;
const HUE_STEP_TENTHS: u16 = 6;

pub struct App {
    display: Digitus16,
    light_sensor: LightSensor,
    hue_tenths: u16,
    status_ticks: u16,
}

impl App {
    pub fn new(peripherals: Peripherals) -> Result<Self> {
        println!(
            "digitus16: SCK GPIO{SPI_CLOCK_GPIO}, MOSI GPIO{SPI_DATA_GPIO}, LDR GPIO{LIGHT_SENSOR_GPIO}, displaying {}",
            core::str::from_utf8(TEXT).unwrap_or("<invalid utf8>")
        );

        let pins = peripherals.pins;
        let display = Digitus16::new(peripherals.spi2, pins.gpio4, pins.gpio6)?;
        let light_sensor = LightSensor::new(peripherals.adc1, pins.gpio0)?;

        Ok(Self {
            display,
            light_sensor,
            hue_tenths: 0,
            status_ticks: 0,
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        let light = self.light_sensor.sample()?;
        self.display
            .show_text(TEXT, self.hue_tenths / 10, light.intensity)?;

        if self.status_ticks == 0 {
            println!(
                "light ADC {:4}, display intensity {:3}",
                light.filtered_raw, light.intensity
            );
        }

        self.status_ticks = (self.status_ticks + 1) % STATUS_INTERVAL_TICKS;
        self.hue_tenths = (self.hue_tenths + HUE_STEP_TENTHS) % 3600;

        Ok(())
    }
}
