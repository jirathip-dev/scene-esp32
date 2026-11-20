use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use serde::{Deserialize, Deserializer};

use super::Coat;

pub(super) fn parse<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Coat>, D::Error> {
    Ok(Option::<String>::deserialize(d)?.and_then(|s| coat(&s)))
}

fn coat(s: &str) -> Option<Coat> {
    let var = |name: &str| {
        s.split(';')
            .find_map(|d| d.trim().strip_prefix(name)?.strip_prefix(':'))
            .map(str::trim)
    };
    Some(Coat {
        coat: hex(var("--c")?)?,
        dark: hex(var("--d")?)?,
        mane: hex(var("--m")?)?,
        blaze: var("--b").and_then(hex),
    })
}

fn hex(s: &str) -> Option<Rgb565> {
    let digits = s.strip_prefix('#').filter(|d| d.len() == 6)?;
    let n = u32::from_str_radix(digits, 16).ok()?;
    Some(Rgb888::new((n >> 16) as u8, (n >> 8) as u8, n as u8).into())
}
