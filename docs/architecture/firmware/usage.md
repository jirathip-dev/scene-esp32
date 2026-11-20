# Usage: how the callers read (#2)

Sketches, not code to copy: the names and the flow are the decision, the bodies are impl's.

## The device: `src/main.rs`

```rust
fn main() -> Result<(), EspError> {
    link_patches();
    EspLogger::initialize_default();

    let board = Board::take()?;
    let (tx, rx) = sync_channel::<Event>(16);
    board::wifi::spawn(board.wifi, tx.clone())?;
    board::controls::spawn(board.controls, tx)?;

    let (mut tft, mut oled, mut led) = (board.tft, board.oled, board.led);
    let mut fb = FrameBuf::new();
    let mut app = App::new();
    loop {
        let event = rx.recv_timeout(app.next_tick()).unwrap_or(Event::Tick);
        app.on(event, uptime());
        if let Some(frame) = app.frame() {
            render::draw(&frame, &mut fb);
            tft.flush(&mut fb)?;
        }
        if let Some(status) = app.status() {
            render::status(&status, &mut oled)?;
            oled.flush()?;
        }
        tft.backlight(app.backlight())?;
        led.show(app.needs_you())?;
    }
}
```

- `Board::take()` is the only place a GPIO number is read; it hands out named parts.
- `app.frame()` and `app.status()` return `Some` only when something changed, so an idle scene sends nothing.
- `tft.flush` sends only the rows `render::draw` touched.
- `app.next_tick()` is 100 ms while a horse walks or it rains, else 1 s.

## The net thread: `src/board/wifi.rs`

```rust
fn run(mut wifi: Wifi, tx: SyncSender<Event>) {
    let mut feed = Feed::new();
    loop {
        let (poll, wait) = feed.take(wifi.get(SCENE_ADDR));
        let _ = tx.send(Event::Poll(poll));
        sleep(wait);
    }
}
```

`wifi.get` joins Wi-Fi when it is down, sends the `GET`, stops reading at 256 KB and returns the body or a `Fail`.
`Feed::take` is the library's: it drops a body equal to the last one, parses a new one, and says how long to
wait. Fresh, stale and lost are not sent: the app works them out from the time of the last good poll.

## A test, on the host

```rust
#[test]
fn button_3_walks_the_rows_that_need_you() {
    let mut app = App::new();
    app.on(Event::Poll(Poll::New(model::parse(SCENE.as_bytes()).unwrap())), secs(1));

    app.on(Event::Input(Input::Press(Button::B3)), secs(2));
    assert_eq!(app.frame().unwrap().caption.text, "#22 · needs you");

    app.on(Event::Input(Input::Press(Button::B3)), secs(3));
    assert_eq!(app.frame().unwrap().caption.text, "acme-api-impl-19 · dead · 2h");
}

#[test]
fn twenty_seconds_without_a_good_poll_is_stale_and_five_minutes_is_lost() {
    let mut link = Link::new();
    link.on(&Poll::New(snapshot()), secs(0));
    link.on(&Poll::Failed(Fail::Timeout), secs(9));

    assert_eq!(link.state(secs(19)), LinkState::Fresh { age: secs(19) });
    assert_eq!(link.state(secs(21)), LinkState::Stale { age: secs(21), why: Fail::Timeout });
    assert_eq!(link.state(secs(301)), LinkState::Lost { why: Fail::Timeout });
}

#[test]
fn the_same_body_twice_is_parsed_once() {
    let mut feed = Feed::new();
    assert!(matches!(feed.take(Ok(SCENE.as_bytes().to_vec())), (Poll::New(_), wait) if wait == secs(5)));
    assert!(matches!(feed.take(Ok(SCENE.as_bytes().to_vec())), (Poll::Same, _)));
    assert!(matches!(feed.take(Err(Fail::Timeout)), (Poll::Failed(Fail::Timeout), wait) if wait == secs(5)));
    assert!(matches!(feed.take(Err(Fail::Timeout)), (_, wait) if wait == secs(10)));
}
```

A layout test compares `format!("{frame:#?}")` with `tests/golden/<repo>.txt`: the display list is the golden, so
it is the same on every OS.

## Screenshots, on the host: `examples/shot.rs`

```rust
fn main() {
    for (name, events) in scripts() {
        let mut app = App::new();
        let mut fb = FrameBuf::new();
        events.into_iter().for_each(|(event, at)| app.on(event, at));
        render::draw(&app.frame().unwrap(), &mut fb);
        png::write(format!("target/shots/{name}.png"), &fb);
    }
}
```

`cargo run --example shot --target <host triple>` writes one PNG per script (each fixture repo, stale, lost,
browse). CI uploads the folder; a PR that changes the look shows it.

## The build script: `build.rs`

```rust
fn main() {
    embuild::espidf::sysenv::output();
    let farm = swag::packs().iter().find(|p| p.name == "farm").unwrap();
    artgen::build(&FARM, farm, out_dir()).unwrap_or_else(|e| panic!("art: {e}"));
    device_config();
}

const FARM: Theme = Theme {
    backdrop: Backdrop { window: [0.0, 66.5, 390.0, 253.5], size: (320, 208), hide: &["sky-night"] },
    slots: &[("--c", Slot::Coat), ("--d", Slot::CoatDark), ("--m", Slot::Mane), ("--b", Slot::Blaze), ("--fk", Slot::Fleet)],
    sprites: &[
        Sprite { name: "WORKING", symbol: "s-scene-edit", size: (66, 50), frames: &[] },
        Sprite { name: "IDLE", symbol: "s-scene-idle", size: (66, 50), frames: &[] },
        Sprite { name: "BALE", symbol: "s-needs", size: (32, 24), frames: &["kf1", "kf2"] },
    ],
};
```

The theme is a typed value, so a wrong field does not compile and no manifest parser is needed. `device_config()`
reads `SCENE_WIFI_SSID`, `SCENE_WIFI_PASS` and `SCENE_ADDR` from the build's environment, fails the build when
`SCENE_ADDR` is set and is not an IPv4 address and port, and passes them on. Unset (as in CI) the firmware builds
and shows "not configured".

## Flashing

```sh
export SCENE_WIFI_SSID=… SCENE_WIFI_PASS=… SCENE_ADDR=192.168.1.20:7779
cargo run --release
cargo run --release --no-default-features --features st7789
```
