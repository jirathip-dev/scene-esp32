/// Characters on one TFT line in the 6×10 font.
pub const TFT_CHARS: usize = 53;
/// Characters on one OLED line in the 6×10 font.
pub const OLED_CHARS: usize = 21;

/// `s` in the font's Latin-1, cut in the middle with `~` to at most `max` characters.
pub fn fit(s: &str, max: usize) -> String {
    let chars: Vec<char> = s
        .chars()
        .map(|c| if u32::from(c) <= 0xff { c } else { '?' })
        .collect();
    if chars.len() <= max {
        return chars.into_iter().collect();
    }
    let Some(keep) = max.checked_sub(1) else {
        return String::new();
    };
    let head = keep - keep / 2;
    let tail = &chars[chars.len() - keep / 2..];
    chars[..head]
        .iter()
        .chain(['~'].iter())
        .chain(tail)
        .collect()
}

/// `s` fitted to `width`, padded so that it reads centred in that width.
pub fn centred(s: &str, width: usize) -> String {
    let s = fit(s, width);
    let pad = (width - s.chars().count()) / 2;
    format!("{:pad$}{s}", "")
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn a_short_text_is_kept_and_a_long_one_is_cut_in_the_middle() {
        assert_eq!(fit("acme", 8), "acme");
        assert_eq!(fit("abcdefghij", 5), "ab~ij");
        assert_eq!(fit("abc", 0), "");
    }
}
