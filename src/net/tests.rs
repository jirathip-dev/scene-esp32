use std::time::Duration;

use super::{read_body, Fail, Feed, Link, LinkState, Poll, MAX_BODY};
use crate::app::{App, Event};
use crate::SCENE;

fn secs(s: u64) -> Duration {
    Duration::from_secs(s)
}

fn good() -> Poll {
    Feed::new().take(Ok(SCENE.as_bytes().to_vec())).0
}

#[test]
fn fresh_up_to_20_s_stale_up_to_5_min_then_lost_and_fresh_on_the_next_good_poll() {
    let mut link = Link::new();
    assert_eq!(link.state(secs(20)), LinkState::Waiting);
    link.on(&Poll::Failed(Fail::NoWifi), secs(15));
    assert_eq!(link.state(secs(21)), LinkState::Lost { why: Fail::NoWifi });

    link.on(&good(), secs(30));
    link.on(&Poll::Failed(Fail::Refused), secs(39));
    assert_eq!(link.state(secs(50)), LinkState::Fresh { age: secs(20) });
    let stale = |age| LinkState::Stale {
        age: secs(age),
        why: Fail::Refused,
    };
    assert_eq!(link.state(secs(51)), stale(21));
    link.on(&Poll::Failed(Fail::Http(404)), secs(100));
    assert_eq!(
        link.state(secs(330)),
        LinkState::Stale {
            age: secs(300),
            why: Fail::Http(404)
        }
    );
    assert_eq!(
        link.state(secs(331)),
        LinkState::Lost {
            why: Fail::Http(404)
        }
    );

    link.on(&Poll::Same, secs(400));
    assert_eq!(link.state(secs(403)), LinkState::Fresh { age: secs(3) });
}

#[test]
fn waits_5_10_20_40_60_60_s_after_failures_in_a_row_and_5_s_after_a_good_poll() {
    let mut feed = Feed::new();
    let waits: Vec<u64> = (0..6)
        .map(|_| feed.take(Err(Fail::Timeout)).1.as_secs())
        .collect();
    assert_eq!(waits, [5, 10, 20, 40, 60, 60]);
    let (poll, wait) = feed.take(Ok(SCENE.as_bytes().to_vec()));
    assert!(matches!(poll, Poll::New(_)));
    assert_eq!(wait, secs(5));
    assert_eq!(feed.take(Err(Fail::NoWifi)).1, secs(5));
    assert_eq!(feed.take(Err(Fail::NoWifi)).1, secs(10));
}

#[test]
fn the_same_body_twice_is_parsed_once_and_redraws_nothing() {
    let (mut feed, mut app) = (Feed::new(), App::new());
    let (poll, _) = feed.take(Ok(SCENE.as_bytes().to_vec()));
    assert!(matches!(poll, Poll::New(_)));
    app.on(Event::Poll(poll), secs(1));
    assert_eq!(app.frame().unwrap().header.title, "acme-api");
    app.status();

    let (poll, wait) = feed.take(Ok(SCENE.as_bytes().to_vec()));
    assert!(matches!(poll, Poll::Same));
    assert_eq!(wait, secs(5));
    app.on(Event::Poll(poll), secs(1));
    assert!(app.frame().is_none());
    assert!(app.status().is_none());
}

#[test]
fn a_body_that_does_not_parse_leaves_the_last_scene_on_screen() {
    let (mut feed, mut app) = (Feed::new(), App::new());
    app.on(
        Event::Poll(feed.take(Ok(SCENE.as_bytes().to_vec())).0),
        secs(1),
    );
    assert_eq!(app.frame().unwrap().header.title, "acme-api");

    let (poll, _) = feed.take(Ok(b"{\"type\":\"screen\",".to_vec()));
    assert!(matches!(poll, Poll::Failed(Fail::BadData)));
    app.on(Event::Poll(poll), secs(1));
    assert!(app.frame().is_none());
    app.on(Event::Tick, secs(22));
    let frame = app.frame().unwrap();
    let horse = |i: &crate::view::Item| matches!(i.what, crate::view::What::Horse { .. });
    assert!(frame.items.iter().any(horse));
    assert_eq!(frame.header.stale.as_deref(), Some("stale 21s"));
    assert_eq!(app.status().unwrap().lines[0], "stale 21s: bad data");
}

#[test]
fn a_body_over_256_kb_is_too_big_and_no_more_than_256_kb_of_it_is_read() {
    let mut read = 0;
    let endless = |buf: &mut [u8]| {
        let n = buf.len().min(1000);
        buf[..n].fill(b' ');
        read += n;
        Ok(n)
    };
    assert_eq!(read_body(endless), Err(Fail::TooBig));
    assert!(read <= MAX_BODY, "read {read} bytes");

    let mut rest = SCENE.as_bytes();
    let body = read_body(|buf| {
        let n = buf.len().min(rest.len()).min(700);
        buf[..n].copy_from_slice(&rest[..n]);
        rest = &rest[n..];
        Ok(n)
    });
    assert_eq!(body.as_deref(), Ok(SCENE.as_bytes()));
}

#[test]
fn lost_shows_no_drover_and_the_reason_on_both_screens() {
    let mut app = App::new();
    app.on(Event::Poll(Poll::Failed(Fail::NotConfigured)), secs(0));
    app.on(Event::Tick, secs(21));
    let frame = app.frame().unwrap();
    assert!(matches!(
        &frame.items[0].what,
        crate::view::What::Big(_, label) if label.trim() == "no drover: not configured"
    ));
    app.on(Event::Poll(Poll::Failed(Fail::NoWifi)), secs(25));
    assert_eq!(app.status().unwrap().lines[0], "no drover: no wifi");
}
