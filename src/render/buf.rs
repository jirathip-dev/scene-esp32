use std::convert::Infallible;
use std::ops::Range;

use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{
    Dimensions, DrawTarget, OriginDimensions, Pixel, PointsIter, Size,
};
use embedded_graphics::primitives::Rectangle;

use crate::view::Frame;

/// The TFT's width in landscape.
pub const WIDTH: u32 = 320;
/// The TFT's height in landscape.
pub const HEIGHT: u32 = 240;

/// The TFT's pixels (in PSRAM on the board) and the rectangles changed since the last flush.
pub struct FrameBuf {
    px: Vec<Rgb565>,
    touched: Vec<Rectangle>,
    pub(crate) drawn: Option<Frame>,
}

impl FrameBuf {
    /// A black buffer whose first draw paints every pixel, as the panel starts with noise.
    pub fn new() -> Self {
        Self {
            px: vec![Rgb565::BLACK; (WIDTH * HEIGHT) as usize],
            touched: Vec::new(),
            drawn: None,
        }
    }

    /// The pixels of `rows`, row after row.
    pub fn rows(&self, rows: Range<u32>) -> &[Rgb565] {
        &self.px[(rows.start * WIDTH) as usize..(rows.end * WIDTH) as usize]
    }

    /// The pixels inside `area`, row after row.
    pub fn pixels(&self, area: Rectangle) -> impl Iterator<Item = Rgb565> + '_ {
        area.points()
            .map(|p| self.px[(p.y as u32 * WIDTH + p.x as u32) as usize])
    }

    /// The rectangles touched since the last call, overlapping ones merged; they are forgotten.
    pub fn take_touched(&mut self) -> Vec<Rectangle> {
        std::mem::take(&mut self.touched)
    }

    /// Marks every pixel touched, so the next flush sends the whole frame.
    pub fn touch_all(&mut self) {
        self.touched = vec![self.bounding_box()];
    }

    pub(crate) fn touch(&mut self, area: Rectangle) {
        let mut area = area.intersection(&self.bounding_box());
        if area.is_zero_sized() {
            return;
        }
        while let Some(i) = self
            .touched
            .iter()
            .position(|r| !r.intersection(&area).is_zero_sized())
        {
            let r = self.touched.swap_remove(i);
            area = union(r, area);
        }
        self.touched.push(area);
    }
}

fn union(a: Rectangle, b: Rectangle) -> Rectangle {
    let (Some(a2), Some(b2)) = (a.bottom_right(), b.bottom_right()) else {
        return if a.is_zero_sized() { b } else { a };
    };
    let tl = a.top_left.component_min(b.top_left);
    Rectangle::with_corners(tl, a2.component_max(b2))
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
