use std::time::Duration;

use scene_esp32::model::{Sky, Weather};
use scene_esp32::net::LinkState;
use scene_esp32::view::{self, Mode};
use shots::{fixture, oled, tft, AT};

const FRESH: LinkState = LinkState::Fresh {
    age: Duration::from_secs(3),
};

#[test]
fn scenes() {
    let snapshot = fixture();
    for (page, repo) in snapshot.ranch.repos.iter().enumerate() {
        let frame = view::frame(Some(&snapshot), &Mode::Ambient { page }, &FRESH, AT);
        tft(&format!("scene-{}", repo.id), &frame);
    }
}

#[test]
fn loading() {
    let ambient = Mode::Ambient { page: 0 };
    tft(
        "loading",
        &view::frame(None, &ambient, &LinkState::Waiting, AT),
    );
}

#[test]
fn storm_fog() {
    let mut storm = fixture();
    storm.ranch.weather = Weather {
        sky: Sky::Storm,
        wind: 1.0,
        fog: 2,
        storm: true,
        line: "storm · fog".into(),
    };
    let ambient = Mode::Ambient { page: 0 };
    tft(
        "storm-fog",
        &view::frame(Some(&storm), &ambient, &FRESH, AT),
    );
}

#[test]
fn oled_idle() {
    let ambient = Mode::Ambient { page: 0 };
    oled(
        "oled-idle",
        &view::status(Some(&fixture()), &ambient, &FRESH, AT),
    );
}
