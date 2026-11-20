# Data shapes (#2)

Types as impl writes them; derives and `serde` attributes are left out. Every field of the wire types defaults
when missing, and every word enum has an `Other` that an unknown word parses to.

## 1. What the device reads

`GET /screens/scene` answers blocks' `Screen`. The device reads this much of it (drover's `scene_screen`, as in
`fixtures/scene.json`):

```jsonc
{ "type": "screen", "blocks": [
  { "type": "section", "header": "snapshot", "blocks": [ { "type": "list", "rows": [
      { "type": "row", "title": "Snapshot", "subtitle": { "text": "as of: not yet known", "tone": "warn" } } ] } ] },
  { "type": "scene", "id": "ranch",
    "load": "calm",                                    // calm | busy | held
    "weather": { "sky": "clear",                       // clear | windy | fog | storm
                 "wind": 0.48, "fog": 0, "storm": false, "line": "load 1.20 · mem normal · swap ?" },
    "repos": [ { "id": "acme-api", "title": "acme-api", "color": "g", "subtitle": "2 queued·1 working",
      "blocks": [ { "id": "h-acme-api-working", "header": "working",   // queued | working | idle | done | blocked/dead
        "rows": [ { "id": "h-acme-api-impl-17", "title": "acme-api-impl-17", "subtitle": "working",
                    "figure": { "state": "working", "act": "impl", "coat": "--c:#b08d5e;--d:#8e6f45;--m:#41321f;--b:#efe7dc" },
                    "tone": "g", "value": "86m", "host": null, "needs": null, "blocked_by": null } ] } ] } ] } ] }
```

Read and ignored: `stream`, `tap`, `tone`, `figure.act`, the screen's `title`, and every field not shown here.

## 2. The model (`model`)

```rust
pub fn parse(body: &[u8]) -> Result<Snapshot, ParseError>;

pub enum ParseError { NotAScreen(serde_json::Error), NoScene, BadScene(serde_json::Error) }

pub struct Snapshot { pub ranch: Ranch, pub notes: Vec<Note>, pub not_shown: u16 }
pub struct Note { pub title: String, pub text: String }

pub struct Ranch { pub load: Load, pub weather: Weather, pub repos: Vec<Repo> }
pub enum Load { Calm, Busy, Held, Other }
pub struct Weather { pub sky: Sky, pub wind: f32, pub fog: u8, pub storm: bool, pub line: String }
pub enum Sky { Clear, Windy, Fog, Storm, Other }

pub struct Repo { pub id: String, pub title: String, pub color: Fleet, pub subtitle: String, pub blocks: Vec<Band> }
pub enum Fleet { L, P, C, G, A, None }
pub struct Band { pub id: String, pub header: String, pub rows: Vec<Horse> }

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
pub struct Figure { pub state: State, pub coat: Option<Coat> }
pub enum State { Queued, Needs, Working, Idle, Parked, Unknown, Done, Blocked, Dead, Other }
pub struct Coat { pub coat: Rgb565, pub dark: Rgb565, pub mane: Rgb565, pub blaze: Option<Rgb565> }
```

- `parse` reads blocks' `Screen`, takes its first `Block::Scene` and reads that block's `more` as a `Ranch`.
  `Section`, `List` and `Row` at the top become `notes`; any other kind adds one to `not_shown`.
- `Coat` is parsed from the `--c:…;--d:…;--m:…;--b:…` string. A `transparent` or missing blaze is `None` and is
  drawn in the coat colour. A string that does not parse is `None`: the pack's default coat.
- A row **needs you** when its state is `Needs`, `Blocked` or `Dead`.

## 3. The display list (`view`)

```rust
pub fn frame(snapshot: Option<&Snapshot>, mode: &Mode, link: &LinkState, now: Duration) -> Frame;
pub fn status(snapshot: Option<&Snapshot>, mode: &Mode, link: &LinkState, now: Duration) -> Status;

pub struct Frame {
    pub header: Header,
    pub veil: Veil,
    pub rain: bool,
    pub items: Vec<Item>,
    pub cursor: Option<usize>,
    pub caption: Caption,
}
pub struct Header { pub title: String, pub fleet: Fleet, pub counts: String, pub page: (u8, u8), pub stale: Option<String> }
pub enum Veil { None, Fog1, Fog2 }
pub struct Item { pub key: String, pub at: Point, pub what: What }
pub enum What {
    Horse { sprite: SpriteId, coat: Option<Coat>, fleet: Fleet, sway: i8 },
    Marker { needs: bool, frame: u8 },
    More(u16),
    Big(SpriteId, String),
}
pub struct Caption { pub text: String, pub tone: Tone }
pub enum Tone { Plain, Mute, Good, Warn, Bad }

pub struct Status { pub lines: [String; 6], pub shift: (i8, i8) }
```

The screen, in pixels:

```
y   0 ┌──────────────────────────────────────────────┐
      │ header: repo, its counts, "stale 45s", ● ○ ○ │ 16
   16 ├──────────────────────────────────────────────┤
      │ backdrop 320×208                             │
      │   queued markers along the fence, up to 12   │
      │   horses 66×50 in two staggered rows of 5:   │
      │   need-you first, then working, idle, done   │
      │   "+N" when a row is full                    │
  224 ├──────────────────────────────────────────────┤
      │ caption: the selected row, else the weather  │ 16
  240 └──────────────────────────────────────────────┘
```

| Row's state | Sprite (swag symbol) | Moves |
|---|---|---|
| working | `s-scene-edit` | sways 4 to 12 px with the wind, bobs 1 px |
| idle, parked, unknown, other | `s-scene-idle` | no |
| done (`FAIL` in its subtitle: `s-scene-fail`) | `s-scene-done` | no |
| blocked | `s-scene-needs` | no |
| dead | `s-scene-fail` | no |
| needs (queued) | `s-needs`, the red bale, at the fence | its two frames |
| queued | a drawn marker at the fence | no |
| no scene yet, or lost | `s-loading`, centred, with the reason | no |
| a repo with no rows | `s-empty`, centred | no |

Text is in a 6×10 Latin-1 font: 53 characters a line on the TFT, 21 by 6 lines on the OLED. `view` cuts a long
title in the middle (`acme-web-impl-48-con…ane-abxx`) and swaps a character the font lacks for `?`. `render`
never measures or cuts text.

## 4. The art (`art`, written by `artgen` at build time)

```rust
pub struct Backdrop { pub size: (u16, u16), pub pixels: &'static [u8] }
pub struct Sprite { pub size: (u16, u16), pub frames: &'static [&'static [u8]], pub palette: &'static [Paint] }
pub enum Paint { Clear, Fixed(Rgb565), Slot(Slot) }
pub enum Slot { Coat, CoatDark, Mane, Blaze, Fleet }
pub enum SpriteId { Working, Idle, Done, Needs, Fail, Loading, Empty, Bale }
pub struct Chrome { pub bg: Rgb565, pub fg: Rgb565, pub mute: Rgb565, pub good: Rgb565, pub warn: Rgb565, pub bad: Rgb565, pub fleet: [Rgb565; 5] }

pub static BACKDROP: Backdrop;
pub static CHROME: Chrome;
pub fn sprite(id: SpriteId) -> &'static Sprite;
```

A sprite's pixel is an index into its palette. `CHROME` comes from swag's `farm` scheme (`--bg`, `--fg`, `--mute`,
`--g`, `--a`, `--r`, `--fk-*`), so the header and caption match the pack.

## 5. Events and state (`app`, `net`, `input`)

```rust
pub enum Event { Poll(Poll), Input(Input), Tick }

pub enum Poll { New(Snapshot), Same, Failed(Fail) }
pub enum Fail { NotConfigured, NoWifi, Refused, Timeout, Http(u16), TooBig, BadData }

impl Feed { pub fn take(&mut self, got: Result<Vec<u8>, Fail>) -> (Poll, Duration); }

pub struct Link { last_good: Option<Duration>, last_fail: Option<Fail> }
impl Link { pub fn on(&mut self, poll: &Poll, now: Duration); pub fn state(&self, now: Duration) -> LinkState; }
pub enum LinkState { Waiting, Fresh { age: Duration }, Stale { age: Duration, why: Fail }, Lost { why: Fail } }

pub enum Input { Press(Button), Tilt(Dir) }
pub enum Button { B1, B2, B3, B4, Stick }
pub enum Dir { Up, Down, Left, Right }

pub enum Mode { Ambient { page: usize }, Browse { repo: String, row: Option<String>, last_input: Duration } }
pub enum Backlight { Full, Dim, Off }
```

`Link` stores two facts and works the state out from the time, so a state that disagrees with the clock cannot
be stored. `Browse` keeps the repo and the row by id, not by position, so a new snapshot does not move the cursor. When the
row is gone the cursor drops; when the repo is gone the first repo shows.
