use embedded_graphics::pixelcolor::{Rgb565, Rgb888};

use super::{parse, Fleet, ParseError, Sky, State};
use crate::SCENE;

fn scene(more: &str) -> String {
    format!(r#"{{"type": "screen", "blocks": [{{"type": "scene", "id": "ranch", {more}}}]}}"#)
}

#[test]
fn the_fixture_gives_five_repos_nineteen_rows_and_one_note() {
    let snapshot = parse(SCENE.as_bytes()).unwrap();
    let rows: Vec<_> = snapshot
        .ranch
        .repos
        .iter()
        .flat_map(|r| &r.blocks)
        .flat_map(|b| &b.rows)
        .collect();
    assert_eq!(snapshot.ranch.repos.len(), 5);
    assert_eq!(rows.len(), 19);
    assert_eq!(snapshot.notes.len(), 1);
    assert_eq!(snapshot.notes[0].text, "as of: not yet known");
    assert_eq!(snapshot.not_shown, 0);
    assert_eq!(snapshot.ranch.weather.wind, 0.48);
    assert_eq!(snapshot.ranch.repos[0].color, Fleet::G);

    let horse = rows.iter().find(|h| h.id == "h-acme-api-impl-17").unwrap();
    assert_eq!(horse.figure.state, State::Working);
    let coat = horse.figure.coat.unwrap();
    assert_eq!(coat.coat, Rgb565::from(Rgb888::new(0xb0, 0x8d, 0x5e)));
    assert_eq!(
        coat.blaze,
        Some(Rgb565::from(Rgb888::new(0xef, 0xe7, 0xdc)))
    );
}

#[test]
fn a_body_that_is_not_a_screen_is_not_a_screen() {
    assert!(matches!(parse(b"[1, 2]"), Err(ParseError::NotAScreen(_))));
}

#[test]
fn a_screen_with_no_scene_has_no_scene() {
    let body = r#"{"type": "screen", "blocks": [{"type": "text", "text": "hi"}]}"#;
    assert!(matches!(parse(body.as_bytes()), Err(ParseError::NoScene)));
}

#[test]
fn a_scene_whose_repos_is_a_number_is_a_bad_scene() {
    let body = scene(r#""repos": 7"#);
    assert!(matches!(
        parse(body.as_bytes()),
        Err(ParseError::BadScene(_))
    ));
}

#[test]
fn unknown_words_are_other_and_unknown_fields_are_ignored() {
    let body = scene(
        r#""weather": {"sky": "hail", "mood": 3},
           "repos": [{"id": "r", "color": "z", "blocks": [{"rows": [
               {"id": "h", "figure": {"state": "galloping", "coat": "--c:red"}, "saddle": true}]}]}]"#,
    );
    let ranch = parse(body.as_bytes()).unwrap().ranch;
    assert_eq!(ranch.weather.sky, Sky::Other);
    assert_eq!(ranch.repos[0].color, Fleet::None);
    let horse = &ranch.repos[0].blocks[0].rows[0];
    assert_eq!(horse.figure.state, State::Other);
    assert_eq!(horse.figure.coat, None);
}

#[test]
fn a_transparent_blaze_is_none() {
    let body = scene(
        r#""repos": [{"blocks": [{"rows": [{"figure": {"coat": "--c:#000000;--d:#000000;--m:#000000;--b:transparent"}}]}]}]"#,
    );
    let ranch = parse(body.as_bytes()).unwrap().ranch;
    assert_eq!(
        ranch.repos[0].blocks[0].rows[0].figure.coat.unwrap().blaze,
        None
    );
}

#[test]
fn other_top_level_kinds_count_as_not_shown() {
    let body = r#"{"type": "screen", "blocks": [{"type": "text", "text": "hi"},
        {"type": "scene", "id": "ranch"}, {"type": "scene", "id": "second"}]}"#;
    assert_eq!(parse(body.as_bytes()).unwrap().not_shown, 2);
}
