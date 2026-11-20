mod board;

use std::time::Instant;

use esp_idf_svc::sys::{heap_caps_get_total_size, EspError, MALLOC_CAP_SPIRAM};
use scene_esp32::app::{App, Event};
use scene_esp32::render::{self, FrameBuf};

fn main() -> Result<(), EspError> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    match scene_esp32::block_count(scene_esp32::SCENE) {
        Ok(n) => log::info!("scene fixture: {n} blocks"),
        Err(e) => log::error!("scene fixture does not parse: {e}"),
    }

    let board::Board {
        mut tft,
        mut oled,
        mut led,
        ..
    } = board::Board::take()?;
    let started = Instant::now();
    let mut fb = FrameBuf::new();
    // SAFETY: reads the size of the PSRAM heap; no pointer is passed.
    let psram = unsafe { heap_caps_get_total_size(MALLOC_CAP_SPIRAM) };
    log::info!(
        "psram {psram} bytes, frame buffer at {:p}",
        fb.rows(0..1).as_ptr()
    );

    let mut app = App::new();
    loop {
        app.on(Event::Tick, started.elapsed());
        if let Some(frame) = app.frame() {
            render::draw(&frame, &mut fb);
            if cfg!(feature = "soak") {
                fb.touch_all();
            }
            tft.flush(&mut fb)?;
        }
        if let Some(status) = app.status() {
            oled.show(&status)?;
        }
        tft.backlight(app.backlight())?;
        led.show(app.needs_you())?;
        std::thread::sleep(app.next_tick());
    }
}
