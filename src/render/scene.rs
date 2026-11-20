use embedded_graphics::draw_target::DrawTargetExt;
use embedded_graphics::mono_font::{iso_8859_1::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};

use super::{FrameBuf, HEIGHT, WIDTH};
use crate::art::{self, Paint, Slot, SpriteId, BACKDROP, CHROME};
use crate::view::{Coat, Fleet, Frame, Item, Tone, Veil, What};

/// The coat a row with none is drawn in: the pack's default.
pub const DEFAULT_COAT: Coat = Coat {
    coat: Rgb565::new(0xc8 >> 3, 0x9a >> 2, 0x56 >> 3),
    dark: Rgb565::new(0xa3 >> 3, 0x7a >> 2, 0x3c >> 3),
    mane: Rgb565::new(0xe8 >> 3, 0xdc >> 2, 0xc0 >> 3),
    blaze: Some(Rgb565::new(0xef >> 3, 0xe7 >> 2, 0xdc >> 3)),
};

const BAND: u32 = 16;
const HEADER: Rectangle = Rectangle::new(Point::zero(), Size::new(WIDTH, BAND));
const SCENE: Rectangle = Rectangle::new(Point::new(0, BAND as i32), Size::new(WIDTH, 208));
const CAPTION: Rectangle = Rectangle::new(Point::new(0, 224), Size::new(WIDTH, BAND));
const FOG: Rgb565 = Rgb565::new(27, 56, 28);
const RAIN: Rgb565 = Rgb565::new(20, 44, 27);

/// Draws `frame` into `fb`, repainting only the rectangles of what changed since the last frame.
pub fn draw(frame: &Frame, fb: &mut FrameBuf) {
    let dirty = match fb.drawn.take() {
        Some(old) if (old.veil, old.rain) == (frame.veil, frame.rain) => changes(&old, frame),
        _ => vec![Rectangle::new(Point::zero(), Size::new(WIDTH, HEIGHT))],
    };
    for area in dirty {
        paint(frame, &mut fb.clipped(&area));
        fb.touch(area);
    }
    fb.drawn = Some(frame.clone());
}

/// The rectangle `item` covers on screen.
pub fn area(item: &Item) -> Rectangle {
    let size = |id| {
        let (w, h) = art::sprite(id).size;
        Size::new(w.into(), h.into())
    };
    match &item.what {
        What::Horse { sprite, sway, .. } => {
            Rectangle::new(item.at + Point::new((*sway).into(), 0), size(*sprite))
        }
        What::Marker { .. } => Rectangle::new(item.at, size(SpriteId::Bale)),
        What::More(_) => Rectangle::new(item.at, Size::new(36, 10)),
        What::Big(sprite, _) => {
            let tall = size(*sprite).height + 16;
            Rectangle::new(Point::new(0, item.at.y), Size::new(WIDTH, tall))
        }
    }
}

fn changes(old: &Frame, new: &Frame) -> Vec<Rectangle> {
    let mut dirty = Vec::new();
    if old.header != new.header {
        dirty.push(HEADER);
    }
    if old.caption != new.caption {
        dirty.push(CAPTION);
    }
    let gone = old.items.iter().filter(|i| !new.items.contains(i));
    let came = new.items.iter().filter(|i| !old.items.contains(i));
    dirty.extend(gone.chain(came).map(area));
    if old.cursor.map(|c| &old.items[c]) != new.cursor.map(|c| &new.items[c]) {
        let cursors = [(old, old.cursor), (new, new.cursor)];
        dirty.extend(
            cursors
                .iter()
                .filter_map(|(f, c)| Some(area(&f.items[(*c)?]))),
        );
    }
    dirty
}

fn paint<D: DrawTarget<Color = Rgb565>>(frame: &Frame, fb: &mut D) {
    let scene = fb.bounding_box().intersection(&SCENE);
    let w = usize::from(BACKDROP.size.0);
    let backdrop = scene.points().map(|p| {
        let i = 2 * ((p.y - SCENE.top_left.y) as usize * w + p.x as usize);
        let be = [BACKDROP.pixels[i], BACKDROP.pixels[i + 1]];
        Pixel(p, Rgb565::from(RawU16::new(u16::from_be_bytes(be))))
    });
    let Ok(()) = fb.draw_iter(backdrop) else {
        return;
    };
    for item in &frame.items {
        let _ = item_into(item, fb);
    }
    let _ = weather(frame, scene, fb);
    let _ = header(frame, fb);
    let tone = match frame.caption.tone {
        Tone::Plain => CHROME.fg,
        Tone::Mute => CHROME.mute,
        Tone::Good => CHROME.good,
        Tone::Warn => CHROME.warn,
        Tone::Bad => CHROME.bad,
    };
    let _ = band(fb, CAPTION, &frame.caption.text, tone);
    if let Some(item) = frame.cursor.map(|c| &frame.items[c]) {
        let _ = area(item)
            .into_styled(PrimitiveStyle::with_stroke(CHROME.warn, 1))
            .draw(fb);
    }
}

fn item_into<D: DrawTarget<Color = Rgb565>>(item: &Item, fb: &mut D) -> Result<(), D::Error> {
    let at = area(item).top_left;
    match &item.what {
        What::Horse {
            sprite,
            coat,
            fleet,
            ..
        } => sprite_into(
            *sprite,
            0,
            at,
            coat.unwrap_or(DEFAULT_COAT),
            fleet_colour(*fleet),
            fb,
        ),
        What::Marker { needs: true, frame } => {
            sprite_into(SpriteId::Bale, *frame, at, DEFAULT_COAT, CHROME.mute, fb)
        }
        What::Marker { needs: false, .. } => {
            let style = PrimitiveStyle::with_fill(CHROME.mute);
            Rectangle::new(at + Point::new(15, 8), Size::new(2, 16))
                .into_styled(style)
                .draw(fb)?;
            let sign = Rectangle::new(at + Point::new(10, 8), Size::new(12, 7));
            sign.into_styled(PrimitiveStyle::with_fill(CHROME.fg))
                .draw(fb)
        }
        What::More(n) => {
            let style = MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(CHROME.fg);
            let style = style.background_color(CHROME.bg).build();
            Text::with_baseline(&format!("+{n}"), at, style, Baseline::Top)
                .draw(fb)
                .map(drop)
        }
        What::Big(sprite, label) => {
            let horse = Point::new(item.at.x, at.y);
            sprite_into(*sprite, 0, horse, DEFAULT_COAT, CHROME.mute, fb)?;
            let style = MonoTextStyle::new(&FONT_6X10, CHROME.fg);
            let below = Point::new(1, at.y + area(item).size.height as i32 - 12);
            Text::with_baseline(label, below, style, Baseline::Top)
                .draw(fb)
                .map(drop)
        }
    }
}

fn sprite_into<D: DrawTarget<Color = Rgb565>>(
    id: SpriteId,
    frame: u8,
    at: Point,
    coat: Coat,
    fleet: Rgb565,
    fb: &mut D,
) -> Result<(), D::Error> {
    let sprite = art::sprite(id);
    let w = usize::from(sprite.size.0);
    let frame = sprite.frames[usize::from(frame) % sprite.frames.len()];
    let px = frame.iter().enumerate().filter_map(|(i, index)| {
        let colour = match sprite.palette[usize::from(*index)] {
            Paint::Clear => return None,
            Paint::Fixed(c) => c,
            Paint::Slot(Slot::Coat) => coat.coat,
            Paint::Slot(Slot::CoatDark) => coat.dark,
            Paint::Slot(Slot::Mane) => coat.mane,
            Paint::Slot(Slot::Blaze) => coat.blaze.unwrap_or(coat.coat),
            Paint::Slot(Slot::Fleet) => fleet,
        };
        Some(Pixel(
            at + Point::new((i % w) as i32, (i / w) as i32),
            colour,
        ))
    });
    fb.draw_iter(px)
}

fn weather<D: DrawTarget<Color = Rgb565>>(
    frame: &Frame,
    scene: Rectangle,
    fb: &mut D,
) -> Result<(), D::Error> {
    let veiled = |p: &Point| match frame.veil {
        Veil::None => false,
        Veil::Fog1 => p.x % 2 == 0 && p.y % 2 == 0,
        Veil::Fog2 => (p.x + p.y) % 2 == 0,
    };
    let fog = scene.points().filter(veiled);
    fb.draw_iter(fog.map(|p| Pixel(p, FOG)))?;
    let rain = scene
        .points()
        .filter(|p| frame.rain && (p.x * 3 + p.y) % 37 == 0 && p.y % 9 < 6);
    fb.draw_iter(rain.map(|p| Pixel(p, RAIN)))
}

fn header<D: DrawTarget<Color = Rgb565>>(frame: &Frame, fb: &mut D) -> Result<(), D::Error> {
    let h = &frame.header;
    let next = band(fb, HEADER, &h.title, CHROME.fg)?;
    let style = MonoTextStyle::new(&FONT_6X10, CHROME.mute);
    let next =
        Text::with_baseline(&h.counts, next + Point::new(6, 0), style, Baseline::Top).draw(fb)?;
    if let Some(stale) = &h.stale {
        let style = MonoTextStyle::new(&FONT_6X10, CHROME.warn);
        Text::with_baseline(stale, next + Point::new(6, 0), style, Baseline::Top).draw(fb)?;
    }
    if h.fleet != Fleet::None {
        let tick = Rectangle::new(Point::new(2, 3), Size::new(4, 10));
        tick.into_styled(PrimitiveStyle::with_fill(fleet_colour(h.fleet)))
            .draw(fb)?;
    }
    let (page, pages) = (i32::from(h.page.0), i32::from(h.page.1));
    for i in 0..pages {
        let dot = Circle::new(Point::new(WIDTH as i32 - 8 * (pages - i), 5), 6);
        let style = match i == page {
            true => PrimitiveStyle::with_fill(CHROME.fg),
            false => PrimitiveStyle::with_stroke(CHROME.mute, 1),
        };
        dot.into_styled(style).draw(fb)?;
    }
    Ok(())
}

fn band<D: DrawTarget<Color = Rgb565>>(
    fb: &mut D,
    area: Rectangle,
    text: &str,
    colour: Rgb565,
) -> Result<Point, D::Error> {
    fb.fill_solid(&area, CHROME.bg)?;
    let style = MonoTextStyle::new(&FONT_6X10, colour);
    let at = area.top_left + Point::new(10, 3);
    Text::with_baseline(text, at, style, Baseline::Top).draw(fb)
}

fn fleet_colour(fleet: Fleet) -> Rgb565 {
    match fleet {
        Fleet::L => CHROME.fleet[0],
        Fleet::P => CHROME.fleet[1],
        Fleet::C => CHROME.fleet[2],
        Fleet::G => CHROME.fleet[3],
        Fleet::A => CHROME.fleet[4],
        Fleet::None => CHROME.mute,
    }
}
