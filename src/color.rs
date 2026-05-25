#[derive(Clone, Copy)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub const OFF: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
    };
}

pub fn hsb_to_rgb(hue: u16, saturation: u8, brightness: u8) -> Rgb {
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
