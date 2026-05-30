use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;

use crate::button::{BUTTON_GPIO, Button};
use crate::digitus16::{DIGITS, Digitus16, SPI_CLOCK_GPIO, SPI_DATA_GPIO};
use crate::light_sensor::{LIGHT_SENSOR_GPIO, LightSensor};
use crate::temperature::{TEMPERATURE_GPIO, TemperatureSensor};

pub const FRAME_DELAY_MS: u32 = 10;

const TEXT: &[u8; DIGITS] = b"FCKAFD";
const STATUS_INTERVAL_TICKS: u16 = 100;
const HUE_STEP_TENTHS: u16 = 6;

pub struct App {
    display: Digitus16,
    light_sensor: LightSensor,
    button: Button,
    temperature_sensor: TemperatureSensor,
    mode: DisplayMode,
    hue_tenths: u16,
    status_ticks: u16,
}

#[derive(Clone, Copy)]
enum DisplayMode {
    Message,
    Temperature,
}

impl DisplayMode {
    fn toggled(self) -> Self {
        match self {
            Self::Message => Self::Temperature,
            Self::Temperature => Self::Message,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Temperature => "temperature",
        }
    }
}

impl App {
    pub fn new(peripherals: Peripherals) -> Result<Self> {
        println!(
            "digitus16: SCK GPIO{SPI_CLOCK_GPIO}, MOSI GPIO{SPI_DATA_GPIO}, LDR GPIO{LIGHT_SENSOR_GPIO}, temp GPIO{TEMPERATURE_GPIO}, button GPIO{BUTTON_GPIO}, displaying {}",
            core::str::from_utf8(TEXT).unwrap_or("<invalid utf8>")
        );

        let pins = peripherals.pins;
        let display = Digitus16::new(peripherals.spi2, pins.gpio4, pins.gpio6)?;
        let light_sensor = LightSensor::new(peripherals.adc1, pins.gpio0)?;
        let button = Button::new(pins.gpio7)?;
        let temperature_sensor = TemperatureSensor::new(pins.gpio10)?;

        Ok(Self {
            display,
            light_sensor,
            button,
            temperature_sensor,
            mode: DisplayMode::Message,
            hue_tenths: 0,
            status_ticks: 0,
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        let light = self.light_sensor.sample()?;
        self.temperature_sensor.poll()?;

        if self.button.was_clicked() {
            self.mode = self.mode.toggled();
            println!("display mode: {}", self.mode.label());
        }

        let text = match self.mode {
            DisplayMode::Message => *TEXT,
            DisplayMode::Temperature => self.temperature_sensor.display_text(),
        };

        self.display
            .show_text(&text, self.hue_tenths / 10, light.intensity)?;

        if self.status_ticks == 0 {
            println!(
                "mode {}, text {}, light ADC {:4}, display intensity {:3}",
                self.mode.label(),
                core::str::from_utf8(&text).unwrap_or("<bad>"),
                light.filtered_raw,
                light.intensity
            );
        }

        self.status_ticks = (self.status_ticks + 1) % STATUS_INTERVAL_TICKS;
        self.hue_tenths = (self.hue_tenths + HUE_STEP_TENTHS) % 3600;

        Ok(())
    }
}
