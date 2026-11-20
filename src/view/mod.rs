use std::time::Duration;

use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

/// What the TFT shows: a title, three colour bars with their names and a footer.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub title: String,
    pub bars: [Bar; 3],
    pub footer: String,
}

/// One colour bar and the name drawn on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub name: &'static str,
    pub color: Rgb565,
}

/// What the OLED shows: six lines of up to 21 characters.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub lines: [String; 6],
}

/// The boot screen: red, green and blue under their names, the panel's name and the uptime.
pub fn boot_frame(uptime: Duration) -> Frame {
    Frame {
        title: format!("{} 320x240", crate::PANEL),
        bars: [
            Bar {
                name: "RED",
                color: Rgb565::RED,
            },
            Bar {
                name: "GREEN",
                color: Rgb565::GREEN,
            },
            Bar {
                name: "BLUE",
                color: Rgb565::BLUE,
            },
        ],
        footer: format!("up {}s", uptime.as_secs()),
    }
}

/// The OLED's boot page.
pub fn boot_status(uptime: Duration) -> Status {
    Status {
        lines: [
            "scene-esp32".into(),
            "booting".into(),
            format!("panel {}", crate::PANEL),
            format!("up {}s", uptime.as_secs()),
            String::new(),
            String::new(),
        ],
    }
}
