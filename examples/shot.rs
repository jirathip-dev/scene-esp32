use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
use embedded_graphics::image::GetPixel;
use embedded_graphics::pixelcolor::raw::{LittleEndian, RawU1};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, Rgb888, RgbColor};
use embedded_graphics::prelude::Point;
use scene_esp32::art::{self, Paint, Slot, SpriteId, BACKDROP, CHROME};
use scene_esp32::model::{self, Sky, Weather};
use scene_esp32::net::{Fail, LinkState};
use scene_esp32::render::{self, FrameBuf, HEIGHT, WIDTH};
use scene_esp32::view::{self, Mode};

type Oled =
    Framebuffer<BinaryColor, RawU1, LittleEndian, 128, 64, { buffer_size::<BinaryColor>(128, 64) }>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("target/shots")?;
    let snapshot = model::parse(scene_esp32::SCENE.as_bytes()).map_err(|e| format!("{e:?}"))?;
    let (at, link) = (
        Duration::from_millis(1200),
        LinkState::Fresh {
            age: Duration::from_secs(3),
        },
    );
    for (page, repo) in snapshot.ranch.repos.iter().enumerate() {
        let frame = view::frame(Some(&snapshot), &Mode::Ambient { page }, &link, at);
        tft(&format!("scene-{}", repo.id), &frame)?;
    }
    let ambient = Mode::Ambient { page: 0 };
    tft(
        "loading",
        &view::frame(None, &ambient, &LinkState::Waiting, at),
    )?;
    let mut storm = snapshot.clone();
    storm.ranch.weather = Weather {
        sky: Sky::Storm,
        wind: 1.0,
        fog: 2,
        storm: true,
        line: "storm · fog".into(),
    };
    tft("storm-fog", &view::frame(Some(&storm), &ambient, &link, at))?;
    let browse = Mode::Browse {
        repo: "acme-api".into(),
        row: Some("h-acme-api-impl-19".into()),
        last_input: at,
    };
    tft("browse", &view::frame(Some(&snapshot), &browse, &link, at))?;
    oled(
        "oled-idle",
        &view::status(Some(&snapshot), &ambient, &link, at),
    )?;
    oled(
        "oled-horse",
        &view::status(Some(&snapshot), &browse, &link, at),
    )?;
    let stale = LinkState::Stale {
        age: Duration::from_secs(45),
        why: Fail::Timeout,
    };
    let lost = LinkState::Lost { why: Fail::NoWifi };
    for (name, link) in [("stale", stale), ("lost", lost)] {
        tft(name, &view::frame(Some(&snapshot), &ambient, &link, at))?;
        oled(
            &format!("oled-{name}"),
            &view::status(Some(&snapshot), &ambient, &link, at),
        )?;
    }
    write_art()
}

fn tft(name: &str, frame: &view::Frame) -> Result<(), Box<dyn std::error::Error>> {
    let mut fb = FrameBuf::new();
    render::draw(frame, &mut fb);
    let rgb = fb.rows(0..HEIGHT).iter().flat_map(|c| rgb888(*c)).collect();
    write(&format!("target/shots/{name}.png"), (WIDTH, HEIGHT), rgb)
}

fn oled(name: &str, status: &view::Status) -> Result<(), Box<dyn std::error::Error>> {
    let mut oled = Oled::new();
    render::status(status, &mut oled)?;
    let points = (0..64).flat_map(|y| (0..128).map(move |x| Point::new(x, y)));
    let on = |p| oled.pixel(p) == Some(BinaryColor::On);
    let rgb = points
        .flat_map(|p| if on(p) { [255; 3] } else { [0; 3] })
        .collect();
    write(&format!("target/shots/{name}.png"), (128, 64), rgb)
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
    let mut coats = vec![render::DEFAULT_COAT];
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
