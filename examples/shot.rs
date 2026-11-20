use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics::image::GetPixel;
use embedded_graphics::pixelcolor::raw::{LittleEndian, RawU1};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, Rgb888, RgbColor};
use embedded_graphics::prelude::Point;
use scene_esp32::app::{App, Event};
use scene_esp32::art::{self, Paint, Slot, SpriteId, BACKDROP, CHROME};
use scene_esp32::model::{self, Coat};
use scene_esp32::render::{self, FrameBuf, HEIGHT, WIDTH};

type Oled =
    Framebuffer<BinaryColor, RawU1, LittleEndian, 128, 64, { buffer_size::<BinaryColor>(128, 64) }>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("target/shots")?;
    let mut app = App::new();
    app.on(Event::Tick, Duration::from_secs(12));

    let mut fb = FrameBuf::new();
    render::draw(&app.frame().ok_or("no boot frame")?, &mut fb);
    let rgb = fb.rows(0..HEIGHT).iter().flat_map(|c| rgb888(*c)).collect();
    write("target/shots/boot.png", (WIDTH, HEIGHT), rgb)?;

    let mut oled = Oled::new();
    render::status(&app.status().ok_or("no boot status")?, &mut oled)?;
    let points = (0..64).flat_map(|y| (0..128).map(move |x| Point::new(x, y)));
    let on = |p| oled.pixel(p) == Some(BinaryColor::On);
    let rgb = points
        .flat_map(|p| if on(p) { [255; 3] } else { [0; 3] })
        .collect();
    write("target/shots/oled-boot.png", (128, 64), rgb)?;
    write_art()
}

const IDS: [SpriteId; 8] = [
    SpriteId::Working,
    SpriteId::Idle,
    SpriteId::Done,
    SpriteId::Needs,
    SpriteId::Fail,
    SpriteId::Loading,
    SpriteId::Empty,
    SpriteId::Bale,
];

fn write_art() -> Result<(), Box<dyn std::error::Error>> {
    let (w, pitch) = (640usize, 70usize);
    let snapshot = model::parse(scene_esp32::SCENE.as_bytes()).map_err(|e| format!("{e:?}"))?;
    let rows = snapshot
        .ranch
        .repos
        .iter()
        .flat_map(|r| &r.blocks)
        .flat_map(|b| &b.rows);
    let mut coats = vec![default_coat()];
    for coat in rows.filter_map(|h| h.figure.coat) {
        if coats.len() < 3 && !coats.contains(&coat) {
            coats.push(coat);
        }
    }
    let h = 208 + coats.len() * 54;
    let mut px = vec![CHROME.bg; w * h];
    let (bw, bh) = (usize::from(BACKDROP.size.0), usize::from(BACKDROP.size.1));
    for (i, be) in BACKDROP.pixels.chunks_exact(2).enumerate() {
        let c = u16::from_be_bytes([be[0], be[1]]);
        px[(i / bw) * w + i % bw] =
            Rgb565::new((c >> 11) as u8, (c >> 5 & 63) as u8, (c & 31) as u8);
    }
    for (row, coat) in coats.iter().enumerate() {
        let frames = IDS
            .iter()
            .map(|id| art::sprite(*id))
            .flat_map(|s| s.frames.iter().map(move |f| (s, f)));
        for (cell, (sprite, frame)) in frames.enumerate() {
            let sw = usize::from(sprite.size.0);
            for (i, index) in frame.iter().enumerate() {
                let (x, y) = (cell * pitch + i % sw, bh + 2 + row * 54 + i / sw);
                let fill = |slot| match slot {
                    Slot::Coat => coat.coat,
                    Slot::CoatDark => coat.dark,
                    Slot::Mane => coat.mane,
                    Slot::Blaze => coat.blaze.unwrap_or(coat.coat),
                    Slot::Fleet => CHROME.fleet[3],
                };
                match sprite.palette[usize::from(*index)] {
                    Paint::Clear => {}
                    Paint::Fixed(c) => px[y * w + x] = c,
                    Paint::Slot(slot) => px[y * w + x] = fill(slot),
                }
            }
        }
    }
    let rgb = px.into_iter().flat_map(rgb888).collect();
    write("target/shots/art.png", (w as u32, h as u32), rgb)
}

fn default_coat() -> Coat {
    let c = |r, g, b| Rgb565::from(Rgb888::new(r, g, b));
    Coat {
        coat: c(0xc8, 0x9a, 0x56),
        dark: c(0xa3, 0x7a, 0x3c),
        mane: c(0xe8, 0xdc, 0xc0),
        blaze: Some(c(0xef, 0xe7, 0xdc)),
    }
}

fn rgb888(c: Rgb565) -> [u8; 3] {
    let c = Rgb888::from(c);
    [c.r(), c.g(), c.b()]
}

fn write(path: &str, (w, h): (u32, u32), rgb: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut png = png::Encoder::new(BufWriter::new(File::create(path)?), w, h);
    png.set_color(png::ColorType::Rgb);
    png.set_depth(png::BitDepth::Eight);
    png.write_header()?.write_image_data(&rgb)?;
    println!("{path}");
    Ok(())
}
