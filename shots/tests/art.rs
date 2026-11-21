use embedded_graphics::pixelcolor::Rgb565;
use scene_esp32::art::{self, Paint, Slot, SpriteId, BACKDROP, CHROME};
use scene_esp32::render;
use shots::{fixture, sheet};

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

#[test]
fn art() {
    let (w, pitch) = (640usize, 70usize);
    let snapshot = fixture();
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
    sheet("art", w, &px);
}
