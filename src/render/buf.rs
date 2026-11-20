use std::convert::Infallible;
use std::ops::Range;

use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{DrawTarget, OriginDimensions, Pixel, RawData, Size};

/// The TFT's width in landscape.
pub const WIDTH: u32 = 320;
/// The TFT's height in landscape.
pub const HEIGHT: u32 = 240;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// The TFT's pixels (in PSRAM on the board) and the span of rows changed since the last flush.
pub struct FrameBuf {
    px: Vec<Rgb565>,
    sums: Vec<u64>,
    touched: Option<Range<u32>>,
}

impl FrameBuf {
    /// A black buffer whose first draw touches every row, as the panel starts with noise.
    pub fn new() -> Self {
        Self {
            px: vec![Rgb565::BLACK; (WIDTH * HEIGHT) as usize],
            sums: vec![0; HEIGHT as usize],
            touched: None,
        }
    }

    /// The pixels of `rows`, row after row.
    pub fn rows(&self, rows: Range<u32>) -> &[Rgb565] {
        &self.px[(rows.start * WIDTH) as usize..(rows.end * WIDTH) as usize]
    }

    /// The rows touched since the last call; they are forgotten.
    pub fn take_touched(&mut self) -> Option<Range<u32>> {
        self.touched.take()
    }

    /// Marks every row touched, so the next flush sends the whole frame.
    pub fn touch_all(&mut self) {
        self.touched = Some(0..HEIGHT);
    }

    pub(crate) fn settle(&mut self) {
        for (y, row) in (0..HEIGHT).zip(self.px.chunks(WIDTH as usize)) {
            let sum = row.iter().fold(FNV_OFFSET, |h, c| {
                (h ^ u64::from(RawU16::from(*c).into_inner())).wrapping_mul(FNV_PRIME)
            });
            if sum != self.sums[y as usize] {
                self.sums[y as usize] = sum;
                self.touched = Some(match self.touched.take() {
                    Some(r) => r.start.min(y)..r.end.max(y + 1),
                    None => y..y + 1,
                });
            }
        }
    }
}

impl OriginDimensions for FrameBuf {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl DrawTarget for FrameBuf {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Infallible>
    where
        I: IntoIterator<Item = Pixel<Rgb565>>,
    {
        for Pixel(p, c) in pixels {
            if let (Ok(x @ 0..WIDTH), Ok(y @ 0..HEIGHT)) = (u32::try_from(p.x), u32::try_from(p.y))
            {
                self.px[(y * WIDTH + x) as usize] = c;
            }
        }
        Ok(())
    }
}
