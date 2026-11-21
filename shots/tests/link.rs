use std::time::Duration;

use scene_esp32::net::{Fail, LinkState};
use scene_esp32::view::{self, Mode};
use shots::{fixture, oled, tft, AT};

#[test]
fn stale_and_lost() {
    let snapshot = fixture();
    let ambient = Mode::Ambient { page: 0 };
    let stale = LinkState::Stale {
        age: Duration::from_secs(45),
        why: Fail::Timeout,
    };
    let lost = LinkState::Lost { why: Fail::NoWifi };
    for (name, link) in [("stale", stale), ("lost", lost)] {
        tft(name, &view::frame(Some(&snapshot), &ambient, &link, AT));
        let status = view::status(Some(&snapshot), &ambient, &link, AT);
        oled(&format!("oled-{name}"), &status);
    }
}
