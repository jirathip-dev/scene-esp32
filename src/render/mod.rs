mod buf;

pub use buf::{FrameBuf, HEIGHT, WIDTH};

use embedded_graphics::mono_font::{iso_8859_1::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};

use crate::view::{Frame, Status};

const BAND: u32 = 24;

/// Draws `frame` into `fb`; only the rows whose pixels changed count as touched.
pub fn draw(frame: &Frame, fb: &mut FrameBuf) {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let bar = (HEIGHT - 2 * BAND) / 3;
    let Ok(()) = fb.clear(Rgb565::BLACK);
    let Ok(_) = Text::with_baseline(&frame.title, Point::new(8, 7), style, Baseline::Top).draw(fb);
    for (i, b) in (0..).zip(&frame.bars) {
        let top = BAND + i * bar;
        let area = Rectangle::new(Point::new(0, top as i32), Size::new(WIDTH, bar));
        let Ok(()) = area
            .into_styled(PrimitiveStyle::with_fill(b.color))
            .draw(fb);
        let at = Point::new(8, (top + bar / 2 - 5) as i32);
        let Ok(_) = Text::with_baseline(b.name, at, style, Baseline::Top).draw(fb);
    }
    let at = Point::new(8, (HEIGHT - BAND + 7) as i32);
    let Ok(_) = Text::with_baseline(&frame.footer, at, style, Baseline::Top).draw(fb);
    fb.settle();
}

/// Draws `status` onto the OLED's 128×64 one-bit target.
pub fn status<D>(status: &Status, oled: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    oled.clear(BinaryColor::Off)?;
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    for (y, line) in (2..).step_by(10).zip(&status.lines) {
        Text::with_baseline(line, Point::new(0, y), style, Baseline::Top).draw(oled)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{draw, FrameBuf, BAND, HEIGHT};
    use crate::view::boot_frame;

    #[test]
    fn drawing_the_same_frame_twice_touches_no_row_the_second_time() {
        let mut fb = FrameBuf::new();
        draw(&boot_frame(Duration::from_secs(3)), &mut fb);
        assert_eq!(fb.take_touched(), Some(0..HEIGHT));

        draw(&boot_frame(Duration::from_secs(3)), &mut fb);
        assert_eq!(fb.take_touched(), None);
    }

    #[test]
    fn a_new_uptime_touches_only_the_footer_rows() {
        let mut fb = FrameBuf::new();
        draw(&boot_frame(Duration::from_secs(3)), &mut fb);
        fb.take_touched();

        draw(&boot_frame(Duration::from_secs(4)), &mut fb);
        let rows = fb.take_touched().unwrap();
        assert!(
            rows.start >= HEIGHT - BAND && rows.end <= HEIGHT,
            "{rows:?}"
        );
    }
}
