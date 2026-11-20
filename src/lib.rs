#[cfg(any(
    all(feature = "ili9341", feature = "st7789"),
    not(any(feature = "ili9341", feature = "st7789"))
))]
compile_error!(
    "pick one panel: feature `ili9341` (the default) or `st7789` (with --no-default-features)"
);

pub mod app;
pub mod art;
pub mod model;
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
