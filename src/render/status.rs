use embedded_graphics::mono_font::{iso_8859_1::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};

use crate::view::Status;

/// Draws `status` onto the OLED's 128×64 one-bit target, shifted against burn-in.
pub fn status<D>(status: &Status, oled: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    oled.clear(BinaryColor::Off)?;
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let (dx, dy) = (i32::from(status.shift.0), i32::from(status.shift.1));
    for (y, line) in (1..).step_by(10).zip(&status.lines) {
        let at = Point::new(dx, y + dy);
        Text::with_baseline(line, at, style, Baseline::Top).draw(oled)?;
    }
    Ok(())
}
