use anyhow::Result;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio5;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{SPI2, SpiBusDriver, SpiDriver, SpiDriverConfig, config};
use esp_idf_hal::units::FromValueType;

const DIGITS: usize = 6;
const LEDS_PER_DIGIT: usize = 16;
const FRAME_LEN: usize = 4 + DIGITS * LEDS_PER_DIGIT * 4 + 8;

// Upstream digitus16's ESP32 driver uses these blonkenclick display pins.
const SPI_CLOCK_GPIO: u8 = 4;
const SPI_DATA_GPIO: u8 = 6;
const TEXT: &[u8; DIGITS] = b"FCKAFD";

// The display documentation recommends a global LED brightness no higher than 15.
const GLOBAL_BRIGHTNESS: u8 = 6;
const COLOR_BRIGHTNESS: u8 = 255;

// Hardware LED order: E-F-D-R-S-T-G-P-U-C-N-M-K-H-B-A.
// Values index font bits in upstream's A-B-C-D-E-F-G-H-K-M-N-P-R-S-T-U order.
const LED_TO_FONT_BIT: [u8; LEDS_PER_DIGIT] =
    [4, 5, 3, 12, 13, 14, 6, 11, 15, 2, 10, 9, 8, 7, 1, 0];

#[derive(Clone, Copy)]
struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl Rgb {
    const OFF: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
    };
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    println!("digitus16: SCK GPIO{SPI_CLOCK_GPIO}, MOSI GPIO{SPI_DATA_GPIO}, displaying HELLO");

    let driver = SpiDriver::new::<SPI2>(
        peripherals.spi2,
        peripherals.pins.gpio4,
        peripherals.pins.gpio6,
        Option::<Gpio5>::None,
        &SpiDriverConfig::new(),
    )?;
    let config = config::Config::new()
        .baudrate(4.MHz().into())
        .write_only(true);
    let mut spi = SpiBusDriver::new(driver, &config)?;

    let mut hue = 0;
    loop {
        spi.write(&render_text(TEXT, hue))?;
        hue = (hue + 3) % 360;
        FreeRtos::delay_ms(50);
    }
}

fn render_text(text: &[u8; DIGITS], base_hue: u16) -> [u8; FRAME_LEN] {
    let mut frame = [0u8; FRAME_LEN];
    let mut cursor = 4;

    // The serial chain runs opposite to the visible left-to-right digit order.
    for hardware_digit in 0..DIGITS {
        let visible_digit = DIGITS - 1 - hardware_digit;
        let mask = glyph(text[visible_digit]);

        for (led, bit) in LED_TO_FONT_BIT.iter().copied().enumerate() {
            let color = if mask & (1u16 << bit) != 0 {
                let hue = (base_hue + visible_digit as u16 * 45 + led as u16 * 3) % 360;
                hsb_to_rgb(hue, 255, COLOR_BRIGHTNESS)
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

fn hsb_to_rgb(hue: u16, saturation: u8, brightness: u8) -> Rgb {
    if saturation == 0 {
        return Rgb {
            red: brightness,
            green: brightness,
            blue: brightness,
        };
    }

    let hue = hue % 360;
    let sector = hue / 60;
    let remainder = (hue % 60) * 255 / 60;
    let saturation = u16::from(saturation);
    let brightness = u16::from(brightness);
    let p = brightness * (255 - saturation) / 255;
    let q = brightness * (255 - saturation * remainder / 255) / 255;
    let t = brightness * (255 - saturation * (255 - remainder) / 255) / 255;

    let (red, green, blue) = match sector {
        0 => (brightness, t, p),
        1 => (q, brightness, p),
        2 => (p, brightness, t),
        3 => (p, q, brightness),
        4 => (t, p, brightness),
        _ => (brightness, p, q),
    };

    Rgb {
        red: red as u8,
        green: green as u8,
        blue: blue as u8,
    }
}

fn glyph(character: u8) -> u16 {
    match character.to_ascii_uppercase() {
        b' ' => 0b0000000000000000,
        b'-' => 0b1000100000000000,
        b'0' => 0b0100010011111111,
        b'1' => 0b0000010000001100,
        b'2' => 0b1000100001110111,
        b'3' => 0b0000100000111111,
        b'4' => 0b1000100010001100,
        b'5' => 0b1001000010110011,
        b'6' => 0b1000100011111011,
        b'7' => 0b0000000000001111,
        b'8' => 0b1000100011111111,
        b'9' => 0b1000100010111111,
        b'A' => 0b1000100011001111,
        b'B' => 0b0010101000111111,
        b'C' => 0b0000000011110011,
        b'D' => 0b0010001000111111,
        b'E' => 0b1000000011110011,
        b'F' => 0b1000000011000011,
        b'G' => 0b0000100011111011,
        b'H' => 0b1000100011001100,
        b'I' => 0b0010001000110011,
        b'J' => 0b0000000001111100,
        b'K' => 0b1001010011000000,
        b'L' => 0b0000000011110000,
        b'M' => 0b0000010111001100,
        b'N' => 0b0001000111001100,
        b'O' => 0b0000000011111111,
        b'P' => 0b1000100011000111,
        b'Q' => 0b0001000011111111,
        b'R' => 0b1001100011000111,
        b'S' => 0b1000100010111011,
        b'T' => 0b0010001000000011,
        b'U' => 0b0000000011111100,
        b'V' => 0b0100010011000000,
        b'W' => 0b0101000011001100,
        b'X' => 0b0101010100000000,
        b'Y' => 0b1000100010111100,
        b'Z' => 0b0100010000110011,
        _ => 0b0010100000000111,
    }
}
