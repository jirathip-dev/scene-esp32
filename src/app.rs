use std::time::Duration;

use crate::input::{Button, Dir, Input};
use crate::model::{Horse, Repo, Snapshot};
use crate::net::{Link, Poll};
use crate::render;
use crate::view::{self, Caption, Frame, Mode, SpriteId, Status, Tone, What};

const ROTATE: Duration = Duration::from_secs(20);
const IDLE: Duration = Duration::from_secs(60);

/// One thing that happened, fed to [`App::on`].
pub enum Event {
    Poll(Poll),
    Input(Input),
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
    backlight: Backlight,
    none_need_you: bool,
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
            backlight: Backlight::Full,
            none_need_you: false,
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
                self.take(s, now);
            }
            Event::Input(input) => self.input(input, now),
            Event::Tick => {
                if let Mode::Browse { last_input, .. } = self.mode {
                    if now.saturating_sub(last_input) >= IDLE {
                        self.ambient(now);
                    }
                }
                if let (Some(s), Mode::Ambient { page }) = (&self.snapshot, &mut self.mode) {
                    if now.saturating_sub(self.turned) >= ROTATE {
                        *page = view::next_page(s, *page);
                        self.turned = now;
                    }
                }
            }
        }
    }

    fn take(&mut self, s: Snapshot, now: Duration) {
        if self.snapshot.is_none() {
            self.mode = Mode::Ambient {
                page: view::first_page(&s),
            };
            self.turned = now;
        }
        if let Mode::Browse { row, .. } = &mut self.mode {
            *row = row
                .take()
                .filter(|id| rows_where(&s, |_| true).any(|(_, h)| &h.id == id));
        }
        let before = self.needing();
        self.snapshot = Some(s);
        if self.backlight == Backlight::Off && self.needing() > before {
            self.backlight = Backlight::Full;
        }
    }

    fn input(&mut self, input: Input, now: Duration) {
        self.none_need_you = false;
        self.backlight = match (self.backlight, input) {
            (Backlight::Off, _) => return self.backlight = Backlight::Full,
            (Backlight::Full, Input::Press(Button::B4)) => return self.backlight = Backlight::Dim,
            (Backlight::Dim, Input::Press(Button::B4)) => return self.backlight = Backlight::Off,
            (lit, _) => lit,
        };
        let Some(s) = &self.snapshot else { return };
        let (repos, page) = (&s.ranch.repos, self.page());
        let Some(here) = repos.get(page) else { return };
        let turn = |by: usize| (repos[(page + by) % repos.len()].id.clone(), None);
        let (repo, row) = match input {
            Input::Press(Button::B1) => turn(repos.len() - 1),
            Input::Press(Button::B2) => turn(1),
            Input::Press(Button::B3) => match self.next_needing(s) {
                Some((r, h)) => (r.id.clone(), Some(h.id.clone())),
                None => (here.id.clone(), self.row().map(String::from)),
            },
            Input::Press(Button::Stick) if self.row().is_some() => return self.ambient(now),
            Input::Press(_) => (here.id.clone(), self.moved(here, None)),
            Input::Tilt(dir) => (here.id.clone(), self.moved(here, Some(dir))),
        };
        self.none_need_you = input == Input::Press(Button::B3) && self.needing() == 0;
        self.mode = Mode::Browse {
            repo,
            row,
            last_input: now,
        };
    }

    fn ambient(&mut self, now: Duration) {
        self.mode = Mode::Ambient { page: self.page() };
        self.turned = now;
    }

    fn page(&self) -> usize {
        match (&self.mode, &self.snapshot) {
            (Mode::Ambient { page }, _) => *page,
            (Mode::Browse { repo, .. }, Some(s)) => s
                .ranch
                .repos
                .iter()
                .position(|r| &r.id == repo)
                .unwrap_or(0),
            (Mode::Browse { .. }, None) => 0,
        }
    }

    fn row(&self) -> Option<&str> {
        match &self.mode {
            Mode::Browse { row, .. } => row.as_deref(),
            Mode::Ambient { .. } => None,
        }
    }

    fn next_needing<'s>(&self, s: &'s Snapshot) -> Option<(&'s Repo, &'s Horse)> {
        let need: Vec<_> = rows_where(s, |h| h.figure.state.needs_you()).collect();
        let at = need
            .iter()
            .position(|(_, h)| Some(h.id.as_str()) == self.row());
        need.get(at.map_or(0, |i| (i + 1) % need.len())).copied()
    }

    fn moved(&self, repo: &Repo, dir: Option<Dir>) -> Option<String> {
        let frame = view::frame(
            self.snapshot.as_ref(),
            &self.mode,
            &self.link.state(self.now),
            self.now,
        );
        let ids: Vec<&String> = repo
            .blocks
            .iter()
            .flat_map(|b| &b.rows)
            .map(|h| &h.id)
            .collect();
        let rows = frame.items.iter().filter(|i| ids.contains(&&i.key));
        let Some((at, dir)) = frame.cursor.zip(dir) else {
            let horses: Vec<_> = rows
                .filter(|i| matches!(i.what, What::Horse { .. }))
                .collect();
            let first = ids.iter().find(|id| horses.iter().any(|i| &&i.key == *id));
            return first.map(|id| id.to_string());
        };
        let from = render::area(&frame.items[at]).center();
        let picks = rows
            .filter(|i| i.key != frame.items[at].key)
            .filter_map(|i| {
                let d = render::area(i).center() - from;
                let (ahead, side) = match dir {
                    Dir::Right => (d.x, d.y),
                    Dir::Left => (-d.x, d.y),
                    Dir::Down => (d.y, d.x),
                    Dir::Up => (-d.y, d.x),
                };
                (ahead > 0 && side.abs() <= ahead).then_some((d.x * d.x + d.y * d.y, i))
            });
        let pick = picks
            .min_by_key(|(far, _)| *far)
            .map_or(&frame.items[at], |(_, i)| i);
        Some(pick.key.clone())
    }

    fn needing(&self) -> usize {
        self.snapshot
            .as_ref()
            .map_or(0, |s| rows_where(s, |h| h.figure.state.needs_you()).count())
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
        let mut frame = view::frame(
            self.snapshot.as_ref(),
            &self.mode,
            &self.link.state(self.now),
            self.now,
        );
        if self.none_need_you {
            frame.caption = Caption {
                text: "nothing needs you".into(),
                tone: Tone::Good,
            };
        }
        changed(&mut self.frame, frame)
    }

    /// The OLED's page, blank while the backlight is off, only when it differs from the last one returned.
    pub fn status(&mut self) -> Option<Status> {
        let status = match self.backlight {
            Backlight::Off => Status {
                lines: Default::default(),
                shift: (0, 0),
            },
            _ => view::status(
                self.snapshot.as_ref(),
                &self.mode,
                &self.link.state(self.now),
                self.now,
            ),
        };
        changed(&mut self.status, status)
    }

    /// The backlight level.
    pub fn backlight(&self) -> Backlight {
        self.backlight
    }

    /// Whether any row needs you, which lights the LED.
    pub fn needs_you(&self) -> bool {
        self.needing() > 0
    }
}

fn rows_where(s: &Snapshot, f: fn(&Horse) -> bool) -> impl Iterator<Item = (&Repo, &Horse)> {
    let rows = s.ranch.repos.iter().flat_map(|r| {
        r.blocks
            .iter()
            .flat_map(move |b| b.rows.iter().map(move |h| (r, h)))
    });
    rows.filter(move |(_, h)| f(h))
}

fn changed<T: Clone + PartialEq>(last: &mut Option<T>, new: T) -> Option<T> {
    (last.as_ref() != Some(&new)).then(|| last.insert(new).clone())
}

#[cfg(test)]
mod tests;
