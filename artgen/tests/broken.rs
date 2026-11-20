use std::path::PathBuf;

use artgen::{Backdrop, Slot, Sprite, Theme};
use swag::Pack;

const SCENE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>"#;

fn theme(backdrop: (u16, u16)) -> Theme {
    Theme {
        backdrop: Backdrop {
            window: [0.0, 0.0, 10.0, 10.0],
            size: backdrop,
            hide: &[],
        },
        slots: &[("--c", Slot::Coat)],
        sprites: &[Sprite {
            name: "BROKEN",
            symbol: "s-broken",
            size: (8, 8),
            frames: &[],
        }],
    }
}

fn build(name: &str, theme: &Theme, sprites: &'static [u8]) -> Result<(), String> {
    let files = vec![
        ("theme.json", &br#"{"scheme": {"dark": "farm"}}"#[..]),
        ("scene.svg", SCENE.as_bytes()),
        ("sprites.svg", sprites),
    ];
    let pack = Pack {
        name: "test",
        files: Vec::leak(files),
    };
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&out).unwrap();
    artgen::build(theme, &pack, &out)
}
#[test]
fn a_slot_and_fixed_colours_build() {
    let svg = br##"<svg><symbol id="s-broken" viewBox="0 0 8 8"><rect width="4" height="8" fill="var(--c)"/><rect x="4" width="4" height="8" fill="var(--q,#102030)"/></symbol></svg>"##;
    build("ok", &theme((10, 10)), svg).unwrap();
}

#[test]
fn a_pixel_that_is_no_fixed_colour_and_no_slot_fails_naming_the_symbol() {
    let svg = br##"<svg><symbol id="s-broken" viewBox="0 0 8 8"><defs><linearGradient id="g"><stop offset="0" stop-color="#000000"/><stop offset="1" stop-color="#ffffff"/></linearGradient></defs><rect width="8" height="8" fill="url(#g)"/></symbol></svg>"##;
    let err = build("pixel", &theme((10, 10)), svg).unwrap_err();
    assert!(err.starts_with("s-broken: pixel ("), "{err}");
    assert!(err.ends_with("no fixed colour and no slot"), "{err}");
}

#[test]
fn a_var_with_no_fallback_and_no_slot_fails_naming_the_symbol() {
    let svg = br##"<svg><symbol id="s-broken" viewBox="0 0 8 8"><rect width="8" height="8" fill="var(--zz)"/></symbol></svg>"##;
    let err = build("var", &theme((10, 10)), svg).unwrap_err();
    assert_eq!(err, "s-broken: var(--zz) has no fallback and no slot");
}

#[test]
fn art_of_256_kb_or_more_fails() {
    let svg = br##"<svg><symbol id="s-broken" viewBox="0 0 8 8"><rect width="8" height="8" fill="var(--c)"/></symbol></svg>"##;
    let err = build("big", &theme((512, 256)), svg).unwrap_err();
    assert_eq!(err, "the art is 262208 bytes, over 262144");
}
