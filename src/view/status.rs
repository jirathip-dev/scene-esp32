use std::time::Duration;

use super::{fit, layout, LinkState, Mode, Status, OLED_CHARS};
use crate::model::{Horse, Snapshot, State};

/// The OLED's page: the selected row in full, else the link, weather, totals and notes.
pub fn status(snapshot: Option<&Snapshot>, mode: &Mode, link: &LinkState, now: Duration) -> Status {
    let minute = now.as_secs() / 60;
    let shift = ((minute % 2) as i8, (minute / 2 % 2) as i8);
    let selected = match (snapshot, mode) {
        (Some(s), Mode::Browse { row: Some(id), .. }) => s
            .ranch
            .repos
            .iter()
            .flat_map(layout::rows)
            .find(|h| &h.id == id),
        _ => None,
    };
    let lines = match (selected, snapshot) {
        (Some(h), _) => horse(h),
        (None, Some(s)) => idle(s, link),
        (None, None) => vec![link_line(link)],
    };
    let mut out: [String; 6] = Default::default();
    for (slot, line) in out.iter_mut().zip(lines) {
        *slot = fit(&line, OLED_CHARS);
    }
    Status { lines: out, shift }
}

fn link_line(link: &LinkState) -> String {
    match link {
        LinkState::Waiting => "joining wifi".into(),
        LinkState::Fresh { age } => format!("fresh {}s", age.as_secs()),
        LinkState::Stale { age, why } => format!("stale {}s: {why}", age.as_secs()),
        LinkState::Lost { why } => format!("no drover: {why}"),
    }
}

fn idle(s: &Snapshot, link: &LinkState) -> Vec<String> {
    let rows = || s.ranch.repos.iter().flat_map(layout::rows);
    let count = |f: fn(State) -> bool| rows().filter(|h| f(h.figure.state)).count();
    let need = count(State::needs_you);
    let work = count(|s| s == State::Working);
    let queue = count(|s| s == State::Queued);
    let mut lines = vec![
        link_line(link),
        s.ranch.weather.line.clone(),
        format!("need {need}·work {work}·queue {queue}"),
    ];
    lines.extend(s.notes.iter().map(|n| format!("{}: {}", n.title, n.text)));
    if s.not_shown > 0 {
        lines.insert(3, format!("+{} not shown", s.not_shown));
    }
    lines
}

fn horse(h: &Horse) -> Vec<String> {
    let title: Vec<char> = h.title.chars().collect();
    let mut lines: Vec<String> = title
        .chunks(OLED_CHARS)
        .take(3)
        .map(|c| c.iter().collect())
        .collect();
    let state = format!("{:?}", h.figure.state).to_lowercase();
    lines.push(match &h.value {
        Some(age) if !age.is_empty() => format!("{state} · {age}"),
        _ => state,
    });
    lines.push(format!("host {}", h.host.as_deref().unwrap_or("-")));
    lines.push(h.subtitle.clone());
    lines
}
