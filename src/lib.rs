#[cfg(any(
    all(feature = "ili9341", feature = "st7789"),
    not(any(feature = "ili9341", feature = "st7789"))
))]
compile_error!(
    "pick one panel: feature `ili9341` (the default) or `st7789` (with --no-default-features)"
);

pub mod app;
pub mod render;
pub mod view;

/// The panel this build drives, picked by the `ili9341` or `st7789` feature.
pub const PANEL: &str = if cfg!(feature = "st7789") {
    "ST7789"
} else {
    "ILI9341"
};

/// drover's demo-fleet export of `/screens/scene`, checked in as the fixture.
pub const SCENE: &str = include_str!("../fixtures/scene.json");

/// Parses a `/screens/scene` body into blocks' screen and counts its top-level blocks.
pub fn block_count(json: &str) -> serde_json::Result<usize> {
    let screen: blocks::Screen = serde_json::from_str(json)?;
    Ok(screen.blocks.map_or(0, |b| b.len()))
}

#[cfg(test)]
mod tests {
    use super::{block_count, SCENE};

    #[test]
    fn the_demo_scene_fixture_has_two_blocks() {
        assert_eq!(block_count(SCENE).unwrap(), 2);
    }

    #[test]
    fn a_screen_whose_blocks_are_not_blocks_is_rejected() {
        assert!(block_count(r#"{"blocks": [{"type": "section", "blocks": 7}]}"#).is_err());
    }
}
