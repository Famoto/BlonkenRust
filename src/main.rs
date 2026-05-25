mod app;
mod color;
mod digitus16;
mod font;
mod light_sensor;

use anyhow::Result;
use app::App;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let mut app = App::new(Peripherals::take()?)?;

    loop {
        app.tick()?;
        FreeRtos::delay_ms(app::FRAME_DELAY_MS);
    }
}
