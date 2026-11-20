use std::time::Duration;

use crate::model::Snapshot;
use crate::net::{Link, Poll};
use crate::view::{self, Frame, Mode, SpriteId, Status, What};

const ROTATE: Duration = Duration::from_secs(20);

/// One thing that happened, fed to [`App::on`].
pub enum Event {
    Poll(Poll),
    Tick,
}

/// How bright the TFT's backlight is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Backlight {
    Full,
    Dim,
    Off,
}

/// The device's state: events in, what to draw out.
pub struct App {
    now: Duration,
    snapshot: Option<Snapshot>,
    link: Link,
    mode: Mode,
    turned: Duration,
    frame: Option<Frame>,
    status: Option<Status>,
}

impl App {
    /// The app at boot, with nothing drawn yet.
    pub fn new() -> Self {
        Self {
            now: Duration::ZERO,
            snapshot: None,
            link: Link::new(),
            mode: Mode::Ambient { page: 0 },
            turned: Duration::ZERO,
            frame: None,
            status: None,
        }
    }

    /// Takes `event`, which happened `now` after boot.
    pub fn on(&mut self, event: Event, now: Duration) {
        self.now = now;
        match event {
            Event::Poll(poll) => {
                self.link.on(&poll, now);
                let Poll::New(s) = poll else { return };
                if self.snapshot.is_none() {
                    self.mode = Mode::Ambient {
                        page: view::first_page(&s),
                    };
                    self.turned = now;
                }
                self.snapshot = Some(s);
            }
            Event::Tick => {
                if let (Some(s), Mode::Ambient { page }) = (&self.snapshot, &mut self.mode) {
                    if now.saturating_sub(self.turned) >= ROTATE {
                        *page = view::next_page(s, *page);
                        self.turned = now;
                    }
                }
            }
        }
    }

    /// How long the loop may wait for an event before it sends a `Tick`: short while a horse walks.
    pub fn next_tick(&self) -> Duration {
        let walking = |f: &Frame| {
            f.items.iter().any(|i| {
                matches!(
                    i.what,
                    What::Horse {
                        sprite: SpriteId::Working,
                        ..
                    }
                )
            })
        };
        match self.frame.as_ref().is_some_and(walking) {
            true => Duration::from_millis(100),
            false => Duration::from_secs(1),
        }
    }

    /// The TFT's frame, only when it differs from the last one returned.
    pub fn frame(&mut self) -> Option<Frame> {
        let frame = view::frame(
            self.snapshot.as_ref(),
            &self.mode,
            &self.link.state(self.now),
            self.now,
        );
        changed(&mut self.frame, frame)
    }

    /// The OLED's page, only when it differs from the last one returned.
    pub fn status(&mut self) -> Option<Status> {
        let status = view::status(
            self.snapshot.as_ref(),
            &self.mode,
            &self.link.state(self.now),
            self.now,
        );
        changed(&mut self.status, status)
    }

    /// The backlight level.
    pub fn backlight(&self) -> Backlight {
        Backlight::Full
    }

    /// Whether any row needs you, which lights the LED.
    pub fn needs_you(&self) -> bool {
        self.snapshot
            .iter()
            .flat_map(|s| &s.ranch.repos)
            .flat_map(|r| &r.blocks)
            .flat_map(|b| &b.rows)
            .any(|h| h.figure.state.needs_you())
    }
}

fn changed<T: Clone + PartialEq>(last: &mut Option<T>, new: T) -> Option<T> {
    (last.as_ref() != Some(&new)).then(|| last.insert(new).clone())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{App, Event};
    use crate::net::Poll;
    use crate::{model, SCENE};

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
}
