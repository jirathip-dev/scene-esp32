use std::time::Duration;

use super::{frame, LinkState, Mode, What};
use crate::model::{self, Snapshot};

fn one_repo(title: &str, rows: &str) -> Snapshot {
    let body = format!(
        r#"{{"type": "screen", "blocks": [{{"type": "scene", "id": "ranch", "repos": [
            {{"id": "r", "title": "{title}", "color": "p", "blocks": [{{"rows": [{rows}]}}]}}]}}]}}"#
    );
    model::parse(body.as_bytes()).unwrap()
}

pub(crate) fn acme_api(now: Duration) -> super::Frame {
    let snapshot = model::parse(crate::SCENE.as_bytes()).unwrap();
    frame(
        Some(&snapshot),
        &Mode::Ambient { page: 0 },
        &LinkState::Waiting,
        now,
    )
}

fn shown(s: &Snapshot) -> super::Frame {
    let mode = Mode::Ambient { page: 0 };
    frame(Some(s), &mode, &LinkState::Waiting, Duration::ZERO)
}

#[test]
fn fourteen_lanes_draw_ten_horses_and_a_plus_four() {
    let rows: Vec<String> = (0..14)
        .map(|i| format!(r#"{{"id": "h{i}", "figure": {{"state": "idle"}}}}"#))
        .collect();
    let frame = shown(&one_repo("r", &rows.join(",")));
    let horses = frame
        .items
        .iter()
        .filter(|i| matches!(i.what, What::Horse { .. }));
    assert_eq!(horses.count(), 10);
    let more: Vec<_> = frame
        .items
        .iter()
        .filter_map(|i| match i.what {
            What::More(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(more, [4]);
}

#[test]
fn a_60_character_title_is_cut_in_the_middle_and_non_latin_1_becomes_a_question_mark() {
    let title = "acme-web-impl-48-contrast-audit-for-every-settings-pane-abxx";
    assert_eq!(title.chars().count(), 60);
    let header = shown(&one_repo(title, "")).header;
    assert_eq!(header.title, "acme-web-imp~s-pane-abxx");

    let header = shown(&one_repo("caf\u{e9} \u{2603}", "")).header;
    assert_eq!(header.title, "caf\u{e9} ?");
}

#[test]
fn each_fixture_repo_matches_its_golden_display_list() {
    let snapshot = model::parse(crate::SCENE.as_bytes()).unwrap();
    let link = LinkState::Fresh {
        age: Duration::ZERO,
    };
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
    for (page, repo) in snapshot.ranch.repos.iter().enumerate() {
        let mode = Mode::Ambient { page };
        let got = format!(
            "{:#?}\n",
            frame(Some(&snapshot), &mode, &link, Duration::ZERO)
        );
        let path = format!("{dir}/{}.txt", repo.id);
        if std::env::var_os("GOLDEN").is_some() {
            std::fs::write(&path, &got).unwrap();
        }
        let want = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(got == want, "{path} differs; GOLDEN=1 rewrites it:\n{got}");
    }
}
