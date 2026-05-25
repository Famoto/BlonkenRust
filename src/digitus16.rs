use anyhow::Result;
use esp_idf_hal::gpio::{Gpio4, Gpio5, Gpio6};
use esp_idf_hal::spi::{SPI2, SpiBusDriver, SpiDriver, SpiDriverConfig, config};
use esp_idf_hal::units::FromValueType;

use crate::color::{Rgb, hsb_to_rgb};
use crate::font::glyph;

pub const DIGITS: usize = 6;
pub const SPI_CLOCK_GPIO: u8 = 4;
pub const SPI_DATA_GPIO: u8 = 6;

const LEDS_PER_DIGIT: usize = 16;
const FRAME_LEN: usize = 4 + DIGITS * LEDS_PER_DIGIT * 4 + 8;
// Recommended upper limit from the digitus16 documentation.
const GLOBAL_BRIGHTNESS: u8 = 15;

// Hardware LED order: E-F-D-R-S-T-G-P-U-C-N-M-K-H-B-A.
// Values index font bits in upstream's A-B-C-D-E-F-G-H-K-M-N-P-R-S-T-U order.
const LED_TO_FONT_BIT: [u8; LEDS_PER_DIGIT] =
    [4, 5, 3, 12, 13, 14, 6, 11, 15, 2, 10, 9, 8, 7, 1, 0];

type OutputSpi = SpiBusDriver<'static, SpiDriver<'static>>;

pub struct Digitus16 {
    spi: OutputSpi,
}

impl Digitus16 {
    pub fn new(spi: SPI2<'static>, clock: Gpio4<'static>, data: Gpio6<'static>) -> Result<Self> {
        let driver = SpiDriver::new::<SPI2>(
            spi,
            clock,
            data,
            Option::<Gpio5>::None,
            &SpiDriverConfig::new(),
        )?;
        let config = config::Config::new()
            .baudrate(4.MHz().into())
            .write_only(true);

        Ok(Self {
            spi: SpiBusDriver::new(driver, &config)?,
        })
    }

    pub fn show_text(&mut self, text: &[u8; DIGITS], base_hue: u16, intensity: u8) -> Result<()> {
        self.spi.write(&render_text(text, base_hue, intensity))?;
        Ok(())
    }
}

fn render_text(text: &[u8; DIGITS], base_hue: u16, intensity: u8) -> [u8; FRAME_LEN] {
    let mut frame = [0u8; FRAME_LEN];
    let mut cursor = 4;

    // The serial chain runs opposite to the visible left-to-right digit order.
    for hardware_digit in 0..DIGITS {
        let visible_digit = DIGITS - 1 - hardware_digit;
        let mask = glyph(text[visible_digit]);

        for (led, bit) in LED_TO_FONT_BIT.iter().copied().enumerate() {
            let color = if mask & (1u16 << bit) != 0 {
                let hue = (base_hue + visible_digit as u16 * 45 + led as u16 * 3) % 360;
                hsb_to_rgb(hue, 255, intensity)
            } else {
                Rgb::OFF
            };

            frame[cursor] = 0b1110_0000 | GLOBAL_BRIGHTNESS;
            frame[cursor + 1] = color.blue;
            frame[cursor + 2] = color.green;
            frame[cursor + 3] = color.red;
            cursor += 4;
        }
    }

    frame[cursor..].fill(0xff);
    frame
}
