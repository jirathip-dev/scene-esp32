use std::time::Duration;

use super::{Debounce, Dir, Stick};

const FULL: u16 = 3000;
const MID: u16 = 1500;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn presses(reads: &[(bool, u64)]) -> usize {
    let mut button = Debounce::default();
    let sequence = reads
        .iter()
        .flat_map(|&(down, for_ms)| std::iter::repeat_n(down, (for_ms / 5) as usize));
    sequence.filter(|&down| button.read(down)).count()
}

#[test]
fn a_5_ms_glitch_sends_nothing() {
    assert_eq!(presses(&[(false, 50), (true, 5), (false, 50)]), 0);
    assert_eq!(
        presses(&[(true, 15), (false, 5), (true, 15), (false, 50)]),
        0
    );
}

#[test]
fn a_30_ms_press_sends_one_press_and_its_release_nothing() {
    assert_eq!(presses(&[(false, 20), (true, 30)]), 1);
    assert_eq!(presses(&[(false, 20), (true, 30), (false, 200)]), 1);
    assert_eq!(presses(&[(true, 30), (false, 30), (true, 30)]), 2);
}

fn tilts(stick: &mut Stick, at: [u16; 2], from: u64, to: u64) -> Vec<(u64, Dir)> {
    (from..to)
        .step_by(5)
        .filter_map(|t| stick.read(at, ms(t)).map(|d| (t, d)))
        .collect()
}

fn pct(p: i32) -> u16 {
    (i32::from(MID) + p * i32::from(FULL / 2) / 100) as u16
}

#[test]
fn the_centre_is_the_mean_of_the_boot_reads_unless_the_stick_was_held() {
    let reads = [[1400, 1600], [1600, 1600]];
    assert_eq!(Stick::learn(&reads, FULL).centre(), [1500, 1600]);
    assert_eq!(
        Stick::learn(&[[2900, 100]; 32], FULL).centre(),
        [1500, 1500]
    );
}

#[test]
fn a_centre_off_by_5_percent_sends_nothing() {
    let mut stick = Stick::learn(&[[MID, MID]; 32], FULL);
    assert!(tilts(&mut stick, [pct(5), pct(-5)], 0, 2000).is_empty());
    assert!(tilts(&mut stick, [pct(38), MID], 2000, 3000).is_empty());
}

#[test]
fn a_held_tilt_sends_at_once_then_at_400_ms_and_every_150_ms_until_under_25_percent() {
    let mut stick = Stick::learn(&[[MID, MID]; 32], FULL);
    let sent: Vec<u64> = tilts(&mut stick, [pct(45), MID], 0, 1000)
        .into_iter()
        .map(|(t, d)| {
            assert_eq!(d, Dir::Right);
            t
        })
        .collect();
    assert_eq!(sent, [0, 400, 550, 700, 850]);
    assert!(tilts(&mut stick, [pct(30), MID], 1000, 2000).len() > 1);
    assert!(tilts(&mut stick, [pct(20), MID], 2000, 3000).is_empty());
    assert_eq!(
        tilts(&mut stick, [MID, pct(-60)], 3000, 3100),
        [(3000, Dir::Up)]
    );
}

#[test]
fn with_both_axes_tilted_the_larger_wins() {
    let mut stick = Stick::learn(&[[MID, MID]; 32], FULL);
    assert_eq!(tilts(&mut stick, [pct(60), pct(-80)], 0, 5), [(0, Dir::Up)]);
    let mut stick = Stick::learn(&[[MID, MID]; 32], FULL);
    assert_eq!(
        tilts(&mut stick, [pct(-90), pct(70)], 0, 5),
        [(0, Dir::Left)]
    );
}
