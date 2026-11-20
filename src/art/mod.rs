use embedded_graphics::pixelcolor::Rgb565;

include!(concat!(env!("OUT_DIR"), "/art.rs"));

/// The scene behind the horses: raw big-endian RGB565, row by row.
pub struct Backdrop {
    pub size: (u16, u16),
    pub pixels: &'static [u8],
}

/// A picture whose pixels index its palette, one byte each; one slice per frame.
pub struct Sprite {
    pub size: (u16, u16),
    pub frames: &'static [&'static [u8]],
    pub palette: &'static [Paint],
}

/// What a sprite's pixel is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Clear,
    Fixed(Rgb565),
    Slot(Slot),
}

/// A colour filled in at draw time from the row's coat or the repo's fleet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot {
    Coat,
    CoatDark,
    Mane,
    Blaze,
    Fleet,
}

/// Which sprite to draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpriteId {
    Working,
    Idle,
    Done,
    Needs,
    Fail,
    Loading,
    Empty,
    Bale,
}

/// The header's and caption's colours, from the pack's scheme; `fleet` in `L, P, C, G, A` order.
pub struct Chrome {
    pub bg: Rgb565,
    pub fg: Rgb565,
    pub mute: Rgb565,
    pub good: Rgb565,
    pub warn: Rgb565,
    pub bad: Rgb565,
    pub fleet: [Rgb565; 5],
}

/// The built sprite for `id`.
pub fn sprite(id: SpriteId) -> &'static Sprite {
    match id {
        SpriteId::Working => &WORKING,
        SpriteId::Idle => &IDLE,
        SpriteId::Done => &DONE,
        SpriteId::Needs => &NEEDS,
        SpriteId::Fail => &FAIL,
        SpriteId::Loading => &LOADING,
        SpriteId::Empty => &EMPTY,
        SpriteId::Bale => &BALE,
    }
}

#[cfg(test)]
mod tests {
    use super::{sprite, Paint, Slot, SpriteId, BACKDROP};

    #[test]
    fn the_backdrop_is_320_by_208_rgb565() {
        assert_eq!(BACKDROP.size, (320, 208));
        assert_eq!(BACKDROP.pixels.len(), 320 * 208 * 2);
    }

    #[test]
    fn every_sprite_indexes_its_palette_and_the_horses_have_coat_slots() {
        use SpriteId::*;
        for id in [Working, Idle, Done, Needs, Fail, Loading, Empty, Bale] {
            let s = sprite(id);
            let (w, h) = s.size;
            for frame in s.frames {
                assert_eq!(frame.len(), usize::from(w) * usize::from(h), "{id:?}");
                assert!(
                    frame.iter().all(|i| usize::from(*i) < s.palette.len()),
                    "{id:?}"
                );
            }
            let coat = s.palette.contains(&Paint::Slot(Slot::Coat));
            assert_eq!(coat, id != Bale, "{id:?}");
        }
        assert_eq!(sprite(Bale).frames.len(), 2);
        assert_eq!(sprite(Working).size, (66, 50));
    }
}
