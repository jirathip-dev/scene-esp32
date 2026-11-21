#[cfg(any(
    all(feature = "ili9341", feature = "st7789"),
    not(any(feature = "ili9341", feature = "st7789"))
))]
compile_error!(
    "pick one panel: feature `ili9341` (the default) or `st7789` (with --no-default-features)"
);

pub mod app;
pub mod art;
pub mod input;
pub mod model;
pub mod net;
pub mod render;
pub mod view;

/// drover's demo-fleet export of `/screens/scene`, checked in as the fixture for the host's tests and shots.
#[cfg(not(target_os = "espidf"))]
pub const SCENE: &str = include_str!("../fixtures/scene.json");
