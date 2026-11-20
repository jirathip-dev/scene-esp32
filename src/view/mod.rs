mod layout;
mod status;
mod text;

use std::time::Duration;

use embedded_graphics::prelude::Point;

pub use crate::art::SpriteId;
pub use crate::model::{Coat, Fleet};
use crate::model::{Repo, Sky, Snapshot};
use crate::net::LinkState;
pub use status::status;
pub use text::{fit, OLED_CHARS, TFT_CHARS};

/// What the TFT shows: a header, the scene's items over the backdrop and a caption.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub header: Header,
    pub veil: Veil,
    pub rain: bool,
    pub items: Vec<Item>,
    pub cursor: Option<usize>,
    pub caption: Caption,
}

/// The top line: the repo, its counts, its page among the repos and how stale the scene is.
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub title: String,
    pub fleet: Fleet,
    pub counts: String,
    pub page: (u8, u8),
    pub stale: Option<String>,
}

/// How thick the fog over the scene is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Veil {
    None,
    Fog1,
    Fog2,
}

/// One thing drawn on the backdrop; `key` follows it from frame to frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub key: String,
    pub at: Point,
    pub what: What,
}

/// What an item is.
#[derive(Clone, Debug, PartialEq)]
pub enum What {
    Horse {
        sprite: SpriteId,
        coat: Option<Coat>,
        fleet: Fleet,
        sway: i8,
    },
    Marker {
        needs: bool,
        frame: u8,
    },
    More(u16),
    Big(SpriteId, String),
}

/// The bottom line: the selected row, else the weather.
#[derive(Clone, Debug, PartialEq)]
pub struct Caption {
    pub text: String,
    pub tone: Tone,
}

/// The colour a caption is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tone {
    Plain,
    Mute,
    Good,
    Warn,
    Bad,
}

/// What the OLED shows: six lines of up to 21 characters, shifted a pixel each minute.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub lines: [String; 6],
    pub shift: (i8, i8),
}

/// Which repo is shown: rotating on its own, or the one the user browses with the row under the cursor.
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Ambient {
        page: usize,
    },
    Browse {
        repo: String,
        row: Option<String>,
        last_input: Duration,
    },
}

/// The TFT's display list for `snapshot` as `mode` shows it, at `now`.
pub fn frame(snapshot: Option<&Snapshot>, mode: &Mode, link: &LinkState, now: Duration) -> Frame {
    if let LinkState::Lost { why } = link {
        return big("scene", SpriteId::Loading, &format!("no drover: {why}"));
    }
    let Some((s, page)) = snapshot.and_then(|s| Some((s, page(s, mode)?))) else {
        return big("scene", SpriteId::Loading, "saddling up");
    };
    let repo = &s.ranch.repos[page];
    let weather = &s.ranch.weather;
    let (items, cursor) = layout::items(repo, row(mode), weather.wind, now);
    let selected = row(mode).and_then(|id| layout::rows(repo).find(|h| h.id == id));
    let caption = selected.map_or_else(
        || Caption {
            text: fit(&weather.line, TFT_CHARS),
            tone: Tone::Mute,
        },
        layout::caption,
    );
    let mut frame = Frame {
        header: header(repo, (page, s.ranch.repos.len()), link),
        veil: match (weather.fog, weather.sky) {
            (0, Sky::Fog) | (1, _) => Veil::Fog1,
            (0, _) => Veil::None,
            _ => Veil::Fog2,
        },
        rain: weather.storm || weather.sky == Sky::Storm,
        items,
        cursor,
        caption,
    };
    if frame.items.is_empty() {
        frame.items = big(&repo.title, SpriteId::Empty, "no lanes").items;
    }
    frame
}

/// The repo Ambient starts on: the first with a row that needs you, else the first with rows.
pub fn first_page(s: &Snapshot) -> usize {
    let repos = &s.ranch.repos;
    let needs = |r: &Repo| layout::rows(r).any(|h| h.figure.state.needs_you());
    let rows = |r: &Repo| layout::rows(r).next().is_some();
    repos
        .iter()
        .position(needs)
        .or_else(|| repos.iter().position(rows))
        .unwrap_or(0)
}

/// The repo after `page` that has rows, wrapping; `page` itself when no other has any.
pub fn next_page(s: &Snapshot, page: usize) -> usize {
    let n = s.ranch.repos.len();
    (1..=n)
        .map(|i| (page + i) % n)
        .find(|&i| layout::rows(&s.ranch.repos[i]).next().is_some())
        .unwrap_or(page)
}

fn page(s: &Snapshot, mode: &Mode) -> Option<usize> {
    let repos = &s.ranch.repos;
    let at = match mode {
        Mode::Ambient { page } => *page,
        Mode::Browse { repo, .. } => repos.iter().position(|r| &r.id == repo).unwrap_or(0),
    };
    (!repos.is_empty()).then(|| at.min(repos.len() - 1))
}

fn row(mode: &Mode) -> Option<&str> {
    match mode {
        Mode::Browse { row, .. } => row.as_deref(),
        Mode::Ambient { .. } => None,
    }
}

fn header(repo: &Repo, (page, pages): (usize, usize), link: &LinkState) -> Header {
    let stale = match link {
        LinkState::Stale { age, .. } => Some(format!("stale {}s", age.as_secs())),
        _ => None,
    };
    let room = 306usize.saturating_sub(pages * 8) / 6;
    let title = fit(&repo.title, room.min(24));
    let used = title.chars().count() + 1 + stale.as_ref().map_or(0, |s| s.len() + 1);
    let counts = fit(&repo.subtitle, room.saturating_sub(used));
    let byte = |n: usize| u8::try_from(n).unwrap_or(u8::MAX);
    Header {
        title,
        fleet: repo.color,
        counts,
        page: (byte(page), byte(pages)),
        stale,
    }
}

fn big(title: &str, sprite: SpriteId, label: &str) -> Frame {
    Frame {
        header: Header {
            title: fit(title, 24),
            fleet: Fleet::None,
            counts: String::new(),
            page: (0, 0),
            stale: None,
        },
        veil: Veil::None,
        rain: false,
        items: vec![Item {
            key: "big".into(),
            at: layout::BIG,
            what: What::Big(sprite, text::centred(label, TFT_CHARS)),
        }],
        cursor: None,
        caption: Caption {
            text: String::new(),
            tone: Tone::Plain,
        },
    }
}

#[cfg(test)]
pub(crate) mod tests;
