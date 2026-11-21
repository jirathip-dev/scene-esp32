use scene_esp32::app::{App, Event};
use scene_esp32::input::{Dir, Input};
use scene_esp32::net::Poll;
use shots::{fixture, oled, tft, AT};

#[test]
fn browse() {
    let mut app = App::new();
    app.on(Event::Poll(Poll::New(fixture())), AT);
    for dir in [Dir::Up, Dir::Left] {
        app.on(Event::Input(Input::Tilt(dir)), AT);
    }
    tft("browse", &app.frame().expect("a browse frame"));
    oled("oled-horse", &app.status().expect("the horse page"));
}
