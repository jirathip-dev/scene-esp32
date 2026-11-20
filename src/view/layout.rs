use std::time::Duration;

use embedded_graphics::prelude::Point;

use super::{fit, Caption, Item, SpriteId, Tone, What, TFT_CHARS};
use crate::model::{Horse, Repo, State};

/// Where a centred big sprite's top-left goes.
pub const BIG: Point = Point::new(127, 70);

const HORSES: usize = 10;
const MARKERS: usize = 12;
const MARKER_Y: i32 = 200;

/// Every row of `repo`, in the order drover sent them.
pub fn rows(repo: &Repo) -> impl Iterator<Item = &Horse> {
    repo.blocks.iter().flat_map(|b| &b.rows)
}

/// The repo's items in drawing order, and the index of the one whose row id is `cursor`.
pub fn items(
    repo: &Repo,
    cursor: Option<&str>,
    wind: f32,
    now: Duration,
) -> (Vec<Item>, Option<usize>) {
    let mut items = markers(repo, now);
    let mut horses: Vec<&Horse> = rows(repo)
        .filter(|h| rank(h.figure.state).is_some())
        .collect();
    horses.sort_by_key(|h| rank(h.figure.state));
    let mut drawn: Vec<(usize, &&Horse)> = horses.iter().take(HORSES).enumerate().collect();
    drawn.sort_by_key(|(slot, _)| slot / 5 == 0);
    let amp = 4 + (8.0 * wind.clamp(0.0, 1.0)).round() as i64;
    let tick = (now.as_millis() / 100) as i64;
    for (slot, h) in drawn {
        let (row, col) = ((slot / 5) as i32, (slot % 5) as i32);
        let at = Point::new(29 - 28 * row + 56 * col, 146 - 50 * row);
        let (sway, bob) = match h.figure.state == State::Working {
            true => walk(tick + 3 * slot as i64, amp),
            false => (0, 0),
        };
        items.push(Item {
            key: h.id.clone(),
            at: at + Point::new(0, bob),
            what: What::Horse {
                sprite: sprite(h),
                coat: h.figure.coat,
                fleet: repo.color,
                sway,
            },
        });
    }
    let at = Point::new(284, 84);
    more(&mut items, "more-horses", horses.len(), HORSES, at);
    let at = cursor.and_then(|id| items.iter().position(|i| i.key == id));
    (items, at)
}

fn markers(repo: &Repo, now: Duration) -> Vec<Item> {
    let queued: Vec<&Horse> = rows(repo)
        .filter(|h| matches!(h.figure.state, State::Queued | State::Needs))
        .collect();
    let mut items: Vec<Item> = queued
        .iter()
        .take(MARKERS)
        .enumerate()
        .map(|(i, h)| Item {
            key: h.id.clone(),
            at: Point::new(4 + 24 * i as i32, MARKER_Y),
            what: What::Marker {
                needs: h.figure.state == State::Needs,
                frame: (now.as_secs() % 2) as u8,
            },
        })
        .collect();
    let at = Point::new(302, 207);
    more(&mut items, "more-queued", queued.len(), MARKERS, at);
    items
}

/// The caption for the selected `h`: its title, subtitle and age, toned by its state.
pub fn caption(h: &Horse) -> Caption {
    let parts = [Some(&h.title), Some(&h.subtitle), h.value.as_ref()];
    let text = parts.into_iter().flatten().filter(|s| !s.is_empty());
    Caption {
        text: fit(
            &text.map(String::as_str).collect::<Vec<_>>().join(" · "),
            TFT_CHARS,
        ),
        tone: match h.figure.state {
            s if s.needs_you() => Tone::Bad,
            State::Working => Tone::Good,
            State::Queued => Tone::Warn,
            _ => Tone::Plain,
        },
    }
}

fn more(items: &mut Vec<Item>, key: &str, count: usize, max: usize, at: Point) {
    if let Some(n) = count.checked_sub(max).filter(|n| *n > 0) {
        items.push(Item {
            key: key.into(),
            at,
            what: What::More(u16::try_from(n).unwrap_or(u16::MAX)),
        });
    }
}

fn rank(state: State) -> Option<u8> {
    match state {
        State::Queued | State::Needs => None,
        State::Blocked | State::Dead => Some(0),
        State::Working => Some(1),
        State::Done => Some(3),
        State::Idle | State::Parked | State::Unknown | State::Other => Some(2),
    }
}

fn sprite(h: &Horse) -> SpriteId {
    match h.figure.state {
        State::Working => SpriteId::Working,
        State::Done if h.subtitle.contains("FAIL") => SpriteId::Fail,
        State::Done => SpriteId::Done,
        State::Blocked => SpriteId::Needs,
        State::Dead => SpriteId::Fail,
        _ => SpriteId::Idle,
    }
}

fn walk(tick: i64, amp: i64) -> (i8, i32) {
    let p = tick.rem_euclid(2 * amp);
    let x = if p < amp { p } else { 2 * amp - p };
    ((x - amp / 2) as i8, (tick / 2 % 2) as i32)
}
