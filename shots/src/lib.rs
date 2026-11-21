use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics::image::GetPixel;
use embedded_graphics::pixelcolor::raw::{LittleEndian, RawU1};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, Rgb888, RgbColor};
use embedded_graphics::prelude::Point;
use scene_esp32::model::{self, Snapshot};
use scene_esp32::render::{self, FrameBuf, HEIGHT, WIDTH};
use scene_esp32::view::{Frame, Status};

type Oled =
    Framebuffer<BinaryColor, RawU1, LittleEndian, 128, 64, { buffer_size::<BinaryColor>(128, 64) }>;

/// The time after boot every still is taken at.
pub const AT: Duration = Duration::from_millis(1200);

/// The checked-in scene, parsed.
pub fn fixture() -> Snapshot {
    model::parse(scene_esp32::SCENE.as_bytes()).expect("the fixture parses")
}

/// Draws `frame` as the TFT does and writes `target/shots/<name>.png` (320×240).
pub fn tft(name: &str, frame: &Frame) {
    let mut fb = FrameBuf::new();
    render::draw(frame, &mut fb);
    let rgb = fb.rows(0..HEIGHT).iter().flat_map(|c| rgb888(*c)).collect();
    write(name, (WIDTH, HEIGHT), rgb);
}

/// Draws `status` as the OLED does and writes `target/shots/<name>.png` (128×64).
pub fn oled(name: &str, status: &Status) {
    let mut oled = Oled::new();
    render::status(status, &mut oled).expect("the OLED draws");
    let points = (0..64).flat_map(|y| (0..128).map(move |x| Point::new(x, y)));
    let on = |p| oled.pixel(p) == Some(BinaryColor::On);
    let rgb = points
        .flat_map(|p| if on(p) { [255; 3] } else { [0; 3] })
        .collect();
    write(name, (128, 64), rgb);
}

/// Writes `px`, `width` pixels to a row, as `target/shots/<name>.png`.
pub fn sheet(name: &str, width: usize, px: &[Rgb565]) {
    let rgb = px.iter().flat_map(|c| rgb888(*c)).collect();
    write(name, (width as u32, (px.len() / width) as u32), rgb);
}

fn rgb888(c: Rgb565) -> [u8; 3] {
    let c = Rgb888::from(c);
    [c.r(), c.g(), c.b()]
}

fn write(name: &str, (w, h): (u32, u32), rgb: Vec<u8>) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/shots");
    let path = format!("{dir}/{name}.png");
    let encode = || -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(dir)?;
        let mut png = png::Encoder::new(BufWriter::new(File::create(&path)?), w, h);
        png.set_color(png::ColorType::Rgb);
        png.set_depth(png::BitDepth::Eight);
        png.write_header()?.write_image_data(&rgb)?;
        Ok(())
    };
    encode().unwrap_or_else(|e| panic!("{path}: {e}"));
}
