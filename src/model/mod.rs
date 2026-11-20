mod coat;

use blocks::{Block, Screen, Toned};
use embedded_graphics::pixelcolor::Rgb565;
use serde::{Deserialize, Deserializer};

/// Why a `/screens/scene` body gives no snapshot.
#[derive(Debug)]
pub enum ParseError {
    NotAScreen(serde_json::Error),
    NoScene,
    BadScene(serde_json::Error),
}

/// One poll's worth of the ranch: the scene, the notes beside it and how many blocks were not shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub ranch: Ranch,
    pub notes: Vec<Note>,
    pub not_shown: u16,
}

/// A row from a top-level section, list or row, shown as a line of text.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub title: String,
    pub text: String,
}

/// drover's scene: the host's load, the weather and the repos.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Ranch {
    pub load: Load,
    pub weather: Weather,
    pub repos: Vec<Repo>,
}

/// How busy the host is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Load {
    Calm,
    Busy,
    Held,
    #[default]
    Other,
}

/// The sky over the ranch and its one-line summary.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Weather {
    pub sky: Sky,
    pub wind: f32,
    pub fog: u8,
    pub storm: bool,
    pub line: String,
}

/// The weather's word.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Sky {
    Clear,
    Windy,
    Fog,
    Storm,
    #[default]
    Other,
}

/// One repo and its bands of lanes.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Repo {
    pub id: String,
    pub title: String,
    pub color: Fleet,
    pub subtitle: String,
    pub blocks: Vec<Band>,
}

/// A repo's fleet colour, by its letter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Fleet {
    L,
    P,
    C,
    G,
    A,
    #[default]
    None,
}

/// A group of rows under one header: queued, working, idle, done, blocked/dead.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Band {
    pub id: String,
    pub header: String,
    pub rows: Vec<Horse>,
}

/// One lane, drawn as a horse.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Horse {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub figure: Figure,
    pub value: Option<String>,
    pub host: Option<String>,
    pub needs: Option<u64>,
    pub blocked_by: Option<u64>,
}

/// What the horse is doing and its coat.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Figure {
    pub state: State,
    #[serde(deserialize_with = "coat::parse")]
    pub coat: Option<Coat>,
}

/// A lane's state word.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum State {
    Queued,
    Needs,
    Working,
    Idle,
    Parked,
    Unknown,
    Done,
    Blocked,
    Dead,
    #[default]
    Other,
}

/// A horse's four colours; no blaze is drawn in the coat colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coat {
    pub coat: Rgb565,
    pub dark: Rgb565,
    pub mane: Rgb565,
    pub blaze: Option<Rgb565>,
}

macro_rules! words {
    ($t:ident { $($w:literal => $v:ident),* $(,)? } else $other:ident) => {
        impl<'de> Deserialize<'de> for $t {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                Ok(match Option::<String>::deserialize(d)?.as_deref() {
                    $(Some($w) => Self::$v,)*
                    _ => Self::$other,
                })
            }
        }
    };
}

words!(Load { "calm" => Calm, "busy" => Busy, "held" => Held } else Other);
words!(Sky { "clear" => Clear, "windy" => Windy, "fog" => Fog, "storm" => Storm } else Other);
words!(Fleet { "l" => L, "p" => P, "c" => C, "g" => G, "a" => A } else None);
words!(State {
    "queued" => Queued, "needs" => Needs, "working" => Working, "idle" => Idle, "parked" => Parked,
    "unknown" => Unknown, "done" => Done, "blocked" => Blocked, "dead" => Dead,
} else Other);

/// Reads a `/screens/scene` body: its first scene becomes the ranch, its sections, lists and rows the notes.
pub fn parse(body: &[u8]) -> Result<Snapshot, ParseError> {
    let screen: Screen = serde_json::from_slice(body).map_err(ParseError::NotAScreen)?;
    let (mut ranch, mut notes, mut not_shown) = (None, Vec::new(), 0u16);
    for block in screen.blocks.unwrap_or_default() {
        match block {
            Block::Scene(scene) if ranch.is_none() => ranch = Some(scene.more),
            block => take_notes(block, &mut notes, &mut not_shown),
        }
    }
    let more = serde_json::Value::Object(ranch.ok_or(ParseError::NoScene)?);
    let ranch = serde_json::from_value(more).map_err(ParseError::BadScene)?;
    Ok(Snapshot {
        ranch,
        notes,
        not_shown,
    })
}

fn take_notes(block: Block, notes: &mut Vec<Note>, not_shown: &mut u16) {
    match block {
        Block::Section(s) => s
            .blocks
            .into_iter()
            .for_each(|b| take_notes(b, notes, not_shown)),
        Block::List(l) => l
            .rows
            .into_iter()
            .for_each(|b| take_notes(b, notes, not_shown)),
        Block::Row(r) => notes.push(Note {
            title: r.title,
            text: match r.subtitle {
                Some(Toned::Text(text)) => text,
                Some(Toned::Label(label)) => label.text,
                None => String::new(),
            },
        }),
        _ => *not_shown = not_shown.saturating_add(1),
    }
}

#[cfg(test)]
mod tests;
