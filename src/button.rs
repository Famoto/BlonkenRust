use anyhow::Result;
use esp_idf_hal::gpio::{Gpio7, Input, PinDriver, Pull};

pub const BUTTON_GPIO: u8 = 7;

const DEBOUNCE_TICKS: u8 = 5;

type ButtonPin = PinDriver<'static, Input>;

pub struct Button {
    pin: ButtonPin,
    stable_down: bool,
    last_raw_down: bool,
    debounce_ticks: u8,
}

impl Button {
    pub fn new(pin: Gpio7<'static>) -> Result<Self> {
        let pin = PinDriver::input(pin, Pull::Up)?;
        let down = pin.is_low();

        Ok(Self {
            pin,
            stable_down: down,
            last_raw_down: down,
            debounce_ticks: 0,
        })
    }

    pub fn was_clicked(&mut self) -> bool {
        let raw_down = self.pin.is_low();

        if raw_down != self.last_raw_down {
            self.last_raw_down = raw_down;
            self.debounce_ticks = DEBOUNCE_TICKS;
            return false;
        }

        if self.debounce_ticks == 0 {
            return false;
        }

        self.debounce_ticks -= 1;

        if self.debounce_ticks == 0 && raw_down != self.stable_down {
            self.stable_down = raw_down;
            return self.stable_down;
        }

        false
    }
}
