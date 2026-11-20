use std::time::Duration;

use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

use super::{area, draw, FrameBuf};
use crate::art::{self, Paint, Slot, CHROME};
use crate::view::tests::acme_api;
use crate::view::{Frame, What};

#[test]
fn drawing_the_same_frame_twice_touches_nothing_the_second_time() {
    let mut fb = FrameBuf::new();
    draw(&acme_api(Duration::ZERO), &mut fb);
    assert_eq!(fb.take_touched(), [fb.bounding_box()]);

    draw(&acme_api(Duration::ZERO), &mut fb);
    assert_eq!(fb.take_touched(), []);
}

#[test]
fn one_tick_with_one_working_horse_touches_only_that_horses_two_rectangles() {
    let (before, after) = (
        acme_api(Duration::from_millis(1000)),
        acme_api(Duration::from_millis(1100)),
    );
    let mut fb = FrameBuf::new();
    draw(&before, &mut fb);
    fb.take_touched();
    draw(&after, &mut fb);

    let horse = |f: &Frame| {
        area(
            f.items
                .iter()
                .find(|i| i.key == "h-acme-api-impl-17")
                .unwrap(),
        )
    };
    let (old, new) = (horse(&before), horse(&after));
    assert_ne!(old, new);
    let both = Rectangle::with_corners(
        old.top_left.component_min(new.top_left),
        old.bottom_right()
            .unwrap()
            .component_max(new.bottom_right().unwrap()),
    );
    let touched = fb.take_touched();
    assert!(!touched.is_empty());
    for r in touched {
        assert_eq!(r.intersection(&both), r, "{r:?} is outside {both:?}");
    }
}

#[test]
fn the_working_horse_wears_its_coat_and_the_repos_fleet_colour() {
    let frame = acme_api(Duration::ZERO);
    let mut fb = FrameBuf::new();
    draw(&frame, &mut fb);

    let item = frame
        .items
        .iter()
        .find(|i| i.key == "h-acme-api-impl-17")
        .unwrap();
    let What::Horse { sprite, .. } = item.what else {
        panic!("{item:?}")
    };
    let sprite = art::sprite(sprite);
    let at = area(item);
    let paint = |slot| {
        let w = usize::from(sprite.size.0);
        let pixels = sprite.frames[0].iter().enumerate();
        let fb = &fb;
        pixels
            .filter(move |(_, i)| sprite.palette[usize::from(**i)] == Paint::Slot(slot))
            .map(move |(i, _)| {
                fb.pixels(Rectangle::new(
                    at.top_left + Point::new((i % w) as i32, (i / w) as i32),
                    Size::new(1, 1),
                ))
                .next()
                .unwrap()
            })
            .collect::<Vec<Rgb565>>()
    };
    let coat = paint(Slot::Coat);
    let blanket = paint(Slot::Fleet);
    assert!(!coat.is_empty() && !blanket.is_empty());
    assert!(coat
        .iter()
        .all(|c| *c == Rgb888::new(0xb0, 0x8d, 0x5e).into()));
    assert!(blanket.iter().all(|c| *c == CHROME.fleet[3]));
}
