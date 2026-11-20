use std::time::Duration;

use super::{App, Backlight, Event};
use crate::input::{Button, Dir, Input};
use crate::model::{self, State};
use crate::net::Poll;
use crate::view::{self, Mode};
use crate::SCENE;

fn secs(s: u64) -> Duration {
    Duration::from_secs(s)
}

fn title(app: &mut App) -> String {
    app.frame().map(|f| f.header.title).unwrap_or_default()
}

#[test]
fn ambient_starts_on_a_repo_that_needs_you_and_turns_every_20_s_past_empty_repos() {
    let mut app = App::new();
    assert!(!app.needs_you());
    assert_eq!(title(&mut app), "scene");

    let snapshot = model::parse(SCENE.as_bytes()).unwrap();
    app.on(Event::Poll(Poll::New(snapshot)), secs(1));
    assert!(app.needs_you());
    assert_eq!(title(&mut app), "acme-api");
    assert_eq!(app.next_tick(), Duration::from_millis(100));

    app.on(Event::Tick, secs(20));
    assert_eq!(app.frame().unwrap().header.title, "acme-api");
    let mut seen = Vec::new();
    for t in [21, 41, 61, 81] {
        app.on(Event::Tick, secs(t));
        seen.push(title(&mut app));
    }
    assert_eq!(seen, ["acme-web", "docs", "others", "acme-api"]);
}

fn fixture() -> model::Snapshot {
    model::parse(SCENE.as_bytes()).unwrap()
}

fn booted() -> App {
    let mut app = App::new();
    app.on(Event::Poll(Poll::New(fixture())), secs(1));
    app
}

fn act(app: &mut App, input: Input, at: u64) {
    app.on(Event::Input(input), secs(at));
}

fn cursor(app: &mut App) -> Option<String> {
    let frame = view::frame(
        app.snapshot.as_ref(),
        &app.mode,
        &app.link.state(app.now),
        app.now,
    );
    frame.cursor.map(|i| frame.items[i].key.clone())
}

fn picked(app: &mut App) -> String {
    let frame = app.frame().unwrap();
    format!("{} | {}", frame.header.title, frame.caption.text)
}

#[test]
fn a_tilt_picks_the_nearest_item_that_way_and_stays_when_there_is_none() {
    let mut app = booted();
    let mut seen = Vec::new();
    for (dir, at) in [
        (Dir::Up, 2),
        (Dir::Right, 3),
        (Dir::Right, 4),
        (Dir::Left, 5),
        (Dir::Left, 6),
        (Dir::Down, 7),
        (Dir::Down, 8),
        (Dir::Right, 9),
    ] {
        act(&mut app, Input::Tilt(dir), at);
        seen.push(cursor(&mut app).unwrap());
    }
    let api = |s: &str| format!("h-acme-api-{s}");
    let want = [
        "impl-17", "rev-15", "rev-15", "impl-17", "impl-19", "q22", "q22", "impl-17",
    ];
    assert_eq!(seen, want.map(api));
    assert!(picked(&mut app).starts_with("acme-api | acme-api-impl-17 · "));
}

#[test]
fn buttons_1_and_2_turn_the_page_and_wrap() {
    let mut app = booted();
    let mut seen = Vec::new();
    for (b, at) in [(Button::B1, 2), (Button::B2, 3), (Button::B2, 4)] {
        act(&mut app, Input::Press(b), at);
        seen.push(app.frame().unwrap().header.title);
    }
    assert_eq!(seen, ["others", "acme-api", "acme-ios"]);
}

#[test]
fn button_3_visits_every_row_that_needs_you_in_drovers_order_across_repos() {
    let mut app = booted();
    let mut seen = Vec::new();
    for at in 2..6 {
        act(&mut app, Input::Press(Button::B3), at);
        seen.push(format!(
            "{} {}",
            app.frame().unwrap().header.title,
            app.row().unwrap()
        ));
    }
    let want = [
        "acme-api h-acme-api-q22",
        "acme-api h-acme-api-impl-19",
        "acme-web h-acme-web-impl-44",
        "acme-api h-acme-api-q22",
    ];
    assert_eq!(seen, want);
}

#[test]
fn button_3_with_nothing_that_needs_you_says_so() {
    let mut calm = fixture();
    let rows = calm.ranch.repos.iter_mut().flat_map(|r| &mut r.blocks);
    rows.flat_map(|b| &mut b.rows)
        .for_each(|h| h.figure.state = State::Idle);
    let mut app = App::new();
    app.on(Event::Poll(Poll::New(calm)), secs(1));
    act(&mut app, Input::Press(Button::B3), 2);
    assert_eq!(app.frame().unwrap().caption.text, "nothing needs you");
}

#[test]
fn the_cursor_follows_its_row_into_a_new_snapshot_and_drops_when_it_is_gone() {
    let mut app = booted();
    act(&mut app, Input::Tilt(Dir::Up), 2);
    let before = app.frame().unwrap();
    let at = |f: &view::Frame| f.items[f.cursor.unwrap()].at;
    let without = |id: &str| {
        let mut s = fixture();
        s.ranch.repos[0]
            .blocks
            .iter_mut()
            .for_each(|b| b.rows.retain(|h| h.id != id));
        s
    };
    app.on(
        Event::Poll(Poll::New(without("h-acme-api-impl-19"))),
        secs(3),
    );
    let moved = app.frame().unwrap();
    assert_eq!(cursor(&mut app).unwrap(), "h-acme-api-impl-17");
    assert_ne!(at(&before), at(&moved));

    app.on(
        Event::Poll(Poll::New(without("h-acme-api-impl-17"))),
        secs(4),
    );
    assert_eq!(cursor(&mut app), None);
    assert_eq!(app.row(), None);
    assert_eq!(app.frame().unwrap().header.title, "acme-api");
}

#[test]
fn sixty_seconds_without_input_returns_to_ambient() {
    let mut app = booted();
    act(&mut app, Input::Press(Button::B2), 2);
    act(&mut app, Input::Tilt(Dir::Up), 3);
    app.on(Event::Tick, secs(62));
    assert!(matches!(app.mode, Mode::Browse { .. }));
    app.on(Event::Tick, secs(63));
    assert_eq!(app.mode, Mode::Ambient { page: 1 });
    assert_eq!(cursor(&mut app), None);
}

#[test]
fn the_stick_press_picks_the_first_horse_or_drops_the_cursor() {
    let mut app = booted();
    act(&mut app, Input::Press(Button::Stick), 2);
    assert_eq!(cursor(&mut app).unwrap(), "h-acme-api-impl-17");
    act(&mut app, Input::Press(Button::Stick), 3);
    assert_eq!(app.mode, Mode::Ambient { page: 0 });
}

#[test]
fn button_4_goes_full_dim_off_and_while_off_an_input_only_wakes() {
    let mut app = booted();
    let mut seen = vec![app.backlight()];
    for at in [2, 3] {
        act(&mut app, Input::Press(Button::B4), at);
        seen.push(app.backlight());
    }
    assert_eq!(seen, [Backlight::Full, Backlight::Dim, Backlight::Off]);
    assert_eq!(app.status().unwrap().lines, <[String; 6]>::default());

    act(&mut app, Input::Tilt(Dir::Up), 4);
    assert_eq!(app.backlight(), Backlight::Full);
    assert_eq!(app.mode, Mode::Ambient { page: 0 });
    assert_ne!(app.status().unwrap().lines[0], "");
}

#[test]
fn a_rise_in_the_count_that_needs_you_wakes() {
    let mut app = booted();
    act(&mut app, Input::Press(Button::B4), 2);
    act(&mut app, Input::Press(Button::B4), 3);
    app.on(Event::Poll(Poll::New(fixture())), secs(4));
    assert_eq!(app.backlight(), Backlight::Off);

    let mut worse = fixture();
    worse.ranch.repos[0].blocks[0].rows[0].figure.state = State::Dead;
    app.on(Event::Poll(Poll::New(worse)), secs(5));
    assert_eq!(app.backlight(), Backlight::Full);
}
