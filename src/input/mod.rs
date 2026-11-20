use std::time::Duration;

const SETTLE: u8 = 4;
const START: i32 = 40;
const END: i32 = 25;
const FIRST_REPEAT: Duration = Duration::from_millis(400);
const REPEAT: Duration = Duration::from_millis(150);

/// One thing the user did with the controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    Press(Button),
    Tilt(Dir),
}

/// The four buttons and the stick's switch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    B1,
    B2,
    B3,
    B4,
    Stick,
}

/// A direction the stick is tilted in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// One button's switch: a change counts after four equal reads, and only the press is sent.
#[derive(Default)]
pub struct Debounce {
    down: bool,
    differ: u8,
}

impl Debounce {
    /// Takes one read, `down` when the switch is closed; true when it makes a press.
    pub fn read(&mut self, down: bool) -> bool {
        if down == self.down {
            self.differ = 0;
            return false;
        }
        self.differ += 1;
        if self.differ < SETTLE {
            return false;
        }
        self.down = down;
        self.differ = 0;
        down
    }
}

/// The stick's two axes in millivolts: a tilt past 40 % of the half-range sends, repeats while held, ends under 25 %.
pub struct Stick {
    centre: [i32; 2],
    half: i32,
    held: Option<(Dir, Duration)>,
}

impl Stick {
    /// Learns the centre as the mean of `reads` per axis; a mean outside 35–65 % of `full` is the middle.
    pub fn learn(reads: &[[u16; 2]], full: u16) -> Self {
        let full = i32::from(full);
        let n = reads.len().max(1) as i32;
        let centre = [0, 1].map(|axis| {
            let mean = reads.iter().map(|r| i32::from(r[axis])).sum::<i32>() / n;
            match (35 * full..=65 * full).contains(&(100 * mean)) {
                true => mean,
                false => full / 2,
            }
        });
        Self {
            centre,
            half: full / 2,
            held: None,
        }
    }

    /// The learned centre, x then y.
    pub fn centre(&self) -> [i32; 2] {
        self.centre
    }

    /// Takes one read of both axes at `now`; the direction when it sends a `Tilt`.
    pub fn read(&mut self, at: [u16; 2], now: Duration) -> Option<Dir> {
        let [dx, dy] = [0, 1].map(|axis| i32::from(at[axis]) - self.centre[axis]);
        let (dir, off) = match dx.abs() >= dy.abs() {
            true => (if dx > 0 { Dir::Right } else { Dir::Left }, dx.abs()),
            false => (if dy > 0 { Dir::Down } else { Dir::Up }, dy.abs()),
        };
        match self.held {
            Some((held, next)) if held == dir && 100 * off >= END * self.half => (now >= next)
                .then(|| {
                    self.held = Some((dir, next + REPEAT));
                    dir
                }),
            _ if 100 * off > START * self.half => {
                self.held = Some((dir, now + FIRST_REPEAT));
                Some(dir)
            }
            _ => {
                self.held = self.held.filter(|_| 100 * off >= END * self.half);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests;
