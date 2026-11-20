use std::time::Duration;

use crate::view::{self, Frame, Status};

/// One thing that happened, fed to [`App::on`].
pub enum Event {
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
    frame: Option<Frame>,
    status: Option<Status>,
}

impl App {
    /// The app at boot, with nothing drawn yet.
    pub fn new() -> Self {
        Self {
            now: Duration::ZERO,
            frame: None,
            status: None,
        }
    }

    /// Takes `event`, which happened `now` after boot.
    pub fn on(&mut self, event: Event, now: Duration) {
        match event {
            Event::Tick => self.now = now,
        }
    }

    /// How long the loop may wait for an event before it sends a `Tick`.
    pub fn next_tick(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// The TFT's frame, only when it differs from the last one returned.
    pub fn frame(&mut self) -> Option<Frame> {
        changed(&mut self.frame, view::boot_frame(self.now))
    }

    /// The OLED's page, only when it differs from the last one returned.
    pub fn status(&mut self) -> Option<Status> {
        changed(&mut self.status, view::boot_status(self.now))
    }

    /// The backlight level.
    pub fn backlight(&self) -> Backlight {
        Backlight::Full
    }

    /// Whether any row needs you, which lights the LED.
    pub fn needs_you(&self) -> bool {
        false
    }
}

fn changed<T: Clone + PartialEq>(last: &mut Option<T>, new: T) -> Option<T> {
    (last.as_ref() != Some(&new)).then(|| last.insert(new).clone())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{App, Event};

    #[test]
    fn a_frame_comes_once_per_second_of_uptime() {
        let mut app = App::new();
        app.on(Event::Tick, Duration::from_millis(1200));
        assert_eq!(app.frame().unwrap().footer, "up 1s");
        assert!(app.frame().is_none());

        app.on(Event::Tick, Duration::from_millis(1900));
        assert!(app.frame().is_none());

        app.on(Event::Tick, Duration::from_millis(2100));
        assert_eq!(app.frame().unwrap().footer, "up 2s");
    }
}
