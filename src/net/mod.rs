use std::fmt;
use std::time::Duration;

use crate::model::{self, Snapshot};

/// The most of a body the device reads; a body that fills it is `TooBig`.
pub const MAX_BODY: usize = 256 * 1024;

const STALE: Duration = Duration::from_secs(20);
const LOST: Duration = Duration::from_secs(300);

/// What a poll of drover brought.
pub enum Poll {
    New(Snapshot),
    Same,
    Failed(Fail),
}

/// Why a poll failed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fail {
    NotConfigured,
    NoWifi,
    Refused,
    Timeout,
    Http(u16),
    TooBig,
    BadData,
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Fail::NotConfigured => f.write_str("not configured"),
            Fail::NoWifi => f.write_str("no wifi"),
            Fail::Refused => f.write_str("refused"),
            Fail::Timeout => f.write_str("timeout"),
            Fail::Http(code) => write!(f, "http {code}"),
            Fail::TooBig => f.write_str("too big"),
            Fail::BadData => f.write_str("bad data"),
        }
    }
}

/// Reads a body through `read` until it ends, stopping at `MAX_BODY`.
pub fn read_body(mut read: impl FnMut(&mut [u8]) -> Result<usize, Fail>) -> Result<Vec<u8>, Fail> {
    let mut body = vec![0; MAX_BODY];
    let mut len = 0;
    while len < MAX_BODY {
        match read(&mut body[len..])? {
            0 => {
                body.truncate(len);
                return Ok(body);
            }
            n => len += n,
        }
    }
    Err(Fail::TooBig)
}

/// Turns bodies into polls: a body equal to the last good one is `Same`, and each poll says how long to wait.
#[derive(Default)]
pub struct Feed {
    last: Option<Vec<u8>>,
    fails: u32,
}

impl Feed {
    /// A feed that has seen no body.
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes what a `GET` returned: the poll for the app and the wait before the next `GET`.
    pub fn take(&mut self, got: Result<Vec<u8>, Fail>) -> (Poll, Duration) {
        let poll = match got {
            Ok(body) if self.last.as_ref() == Some(&body) => Poll::Same,
            Ok(body) => match model::parse(&body) {
                Ok(s) => {
                    self.last = Some(body);
                    Poll::New(s)
                }
                Err(_) => Poll::Failed(Fail::BadData),
            },
            Err(fail) => Poll::Failed(fail),
        };
        self.fails = match poll {
            Poll::Failed(_) => self.fails + 1,
            _ => 0,
        };
        let secs = 5u64 << self.fails.saturating_sub(1).min(4);
        (poll, Duration::from_secs(secs.min(60)))
    }
}

/// How current the scene is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinkState {
    Waiting,
    Fresh { age: Duration },
    Stale { age: Duration, why: Fail },
    Lost { why: Fail },
}

/// The time of the last good poll and the last failure since; the state is read off them.
#[derive(Default)]
pub struct Link {
    last_good: Option<Duration>,
    last_fail: Option<Fail>,
}

impl Link {
    /// A link that has had no poll.
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes `poll`, which came `now` after boot.
    pub fn on(&mut self, poll: &Poll, now: Duration) {
        match poll {
            Poll::New(_) | Poll::Same => {
                *self = Self {
                    last_good: Some(now),
                    last_fail: None,
                }
            }
            Poll::Failed(fail) => self.last_fail = Some(*fail),
        }
    }

    /// The link's state at `now`.
    pub fn state(&self, now: Duration) -> LinkState {
        let why = self.last_fail.unwrap_or(Fail::Timeout);
        match self.last_good.map(|t| now.saturating_sub(t)) {
            None if now > STALE => LinkState::Lost { why },
            None => LinkState::Waiting,
            Some(age) if age > LOST => LinkState::Lost { why },
            Some(age) if age > STALE => LinkState::Stale { age, why },
            Some(age) => LinkState::Fresh { age },
        }
    }
}

#[cfg(test)]
mod tests;
