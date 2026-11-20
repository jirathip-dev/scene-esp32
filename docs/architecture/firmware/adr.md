# ADR: firmware structure (#2)

## Summary

Decision: One library that builds and tests on a laptop (model, link, input, view, render, art) and a thin device
layer for pins, screens and Wi-Fi. It polls drover every 5 s, marks the scene stale after 20 s and swaps it for
"no drover" after 5 min. The build turns swag's pinned farm pack into pixels; the chip never reads SVG.
Slices: display bring-up, model and art, scene render, Wi-Fi polling, input: five issues in that order.
Open questions: the Wi-Fi password is baked into the firmware from build-time variables (readable from the board);
the plan adds six libraries (section 9). Approve both, or say which to change.

## Question

How is the firmware split into modules, what does it read and keep in memory, where does its art come from, and
what do the two screens and the controls do, so that milestones 2–6 can be built one PR at a time without
re-deciding any of it?

## What was read

- `fixtures/scene.json` (drover's demo export, 13 KB): a `screen` with one `section` (the snapshot's age) and one
  `scene` block. blocks' `Scene` type knows only `id`, `label` and `lanes`; drover's `load`, `weather` and `repos`
  arrive in its `more` map. The sections inside a repo carry `rows`, not `blocks`, so they are drover's shape, not
  blocks' `Section`.
- drover#1497 (the LAN listener, open): plain HTTP, `GET /screens/scene` only, no stream, no actions, and a `Host`
  that must be an IP literal.
- swag at `bf9a2c6b` (its `staging` head): `packs/farm/` holds `scene.svg` (390×320) and `sprites.svg`. The six
  `s-scene-*` horses leave `--c`, `--d`, `--m`, `--b` open, exactly the four colours a row's `figure.coat` sends;
  `--fk` is the fleet colour. The night colours are not in the pack (they live in drover's page CSS).
- The board: 16 MB flash, 8 MB PSRAM, and the pin map in the README.

## Decisions

### 1. Modules

A library (`src/lib.rs`) with no esp-idf in it, and a device layer (`src/main.rs`, `src/board/`) that is the only
code touching hardware. The library is where the logic and the tests are; the diagram and the call rules are in
[README.md](README.md).

| Option | Verdict |
|---|---|
| Everything in the binary, tested on the board | rejected: CI has no board, so nothing would be proved by a run |
| One crate per module | rejected: seven manifests for about 2,000 lines; `arch.toml` bans enforce the same edges |
| **A pure library and a thin device layer in one crate** | chosen: it is the split the skeleton already has |

`view` turns the model into a display list (what goes where, every text already cut to fit); `render` only draws
that list. So the layout is tested as text, and `render` never sees the model.

Core kinds drawn: `scene` on the TFT. A top-level `section`, `list` or `row` becomes a line of text on the OLED.
Any other kind is counted and shown as "+N not shown", so nothing vanishes silently.

### 2. Data: the poll

- `GET http://<address>/screens/scene` every **5 s**, 4 s timeout, a new connection each time. The address is an
  IPv4 address and port, so the `Host` header passes the listener's guard with no `device_hosts` entry.
- The body is capped at **256 KB** (20 times the fixture); a larger one is a failed poll. A body whose hash equals
  the last one is not parsed and nothing is redrawn.
- Parsing happens once, at the edge: the bytes become blocks' `Screen`, its `scene` block's `more` becomes the
  device's `Ranch` types ([shapes.md](shapes.md)). An unknown word parses as `Other`, an unknown field is ignored,
  and a body that is no screen or has no scene is a failed poll: the last good scene stays.
- Failure: retry after 5, 10, 20, 40, then 60 s. **Stale** when the last good scene is over 20 s old: it stays on
  screen with an amber "stale 45s" and the reason on the OLED. **Lost** after 5 min, or after 20 s with no
  scene yet: the loading horse and "no drover", never old data shown as current ([states.md](states.md)).

| Option | Verdict |
|---|---|
| The `/stream` route | not offered by the listener |
| 1 s polling | rejected: the scene's ages change by the minute, and it costs drover five times the work |
| **5 s polling with a hash check** | chosen |
| 30 s polling | rejected: a horse that needs you would show half a minute late |

### 3. Memory

| What | Where | Bytes |
|---|---|---|
| Frame buffer, 320×240 RGB565 | PSRAM | 153,600 |
| Response body (cap) | PSRAM | 262,144 |
| Parsed `Screen`, only while parsing (about 10× the body; M3 measures it) | PSRAM | up to about 2.6 M |
| `Ranch`: the new one and the one on screen | PSRAM | up to about 0.5 M |
| Art: one backdrop 320×208 RGB565, seven horses 66×50, a bale | flash, read in place | about 158,000 |
| SPI send chunk, OLED buffer | internal RAM | 4,096 + 1,024 |
| Stacks: net 32 KB (the parse is deep), input 4 KB, main 8 KB | internal RAM | 45,056 |

Worst case about 3.5 MB of the 8 MB PSRAM; a normal poll stays under 0.5 MB. The art costs no RAM: a dirty
rectangle is restored by copying from flash, so there is no second frame buffer. `sdkconfig.defaults` turns on
octal PSRAM with `malloc` falling through to it, 16 MB flash, and a 4 MB app partition (no OTA).

### 4. Art

- **Source and pin:** swag's `packs/farm/`, taken as a Cargo build-dependency pinned by full commit, the same way
  `blocks` is pinned. swag has no dependencies, so the two pins cannot clash. `legacy-board/` is not used: swag
  removes it once drover's own art moves.
- **When:** `build.rs` calls `artgen` (a small host-only crate in this repo), which rasterizes with `resvg` and
  writes the pixels to the build folder; the firmware includes them. Nothing generated is committed.
- **Backdrop:** the bottom 253.5 units of `scene.svg` at 320×208, day only, raw RGB565.
- **Sprites:** each horse at 66×50 as one byte per pixel into a small palette whose entries are clear, a fixed
  colour, or a **slot** (coat, coat dark, mane, blaze, fleet). The chip fills the slots from the row's
  `figure.coat` and the repo's colour as it draws, so any coat drover sends works with no new art. Edges are not
  anti-aliased: every pixel is exactly one paint, and `artgen` fails the build on a pixel it cannot place.
- **CSS variables:** `resvg` has none, so `artgen` replaces each `var()` with its fallback or a slot marker first,
  and fails on one that is neither.

| Option | Verdict |
|---|---|
| Parse SVG on the chip | ruled out by the issue |
| Commit generated pixel files | rejected: binary files in git, and a generated-file gate that needs byte-equal output on every OS |
| swag ships RGB565 | rejected: swag is data only, with no build step by its own rules |
| **Rasterize in `build.rs` from the pinned pack** | chosen |

### 5. Display

- **Driver:** the `mipidsi` crate, which has both panels. `ili9341` and `st7789` are Cargo features; `ili9341` is
  in `default`, the other is `--no-default-features --features st7789`. Both or neither is one `compile_error!`.
  The feature picks the model, colour order (BGR / RGB) and inversion (off / on) in `src/board/tft.rs` and nowhere
  else. Landscape, 320×240. Detecting the panel at run time was rejected: the TFT's data-out pin is not wired.
- **SPI:** SPI2 on its native pins (the README's pin map), mode 0, write-only, **20 MHz**. The chip divides
  80 MHz by a whole number, so the choices in range are 20 and 26.7 MHz. A full frame is 61 ms at 20 and 46 ms at
  26.7; the scene redraws only what changed, so the faster clock buys nothing visible and costs margin on jumper
  wires. One constant; M2's soak test proves it.
- The touch chip shares the bus and is unused: its chip-select is held high. The backlight is PWM on GPIO 21.

### 6. Input

| Control | In the scene |
|---|---|
| Joystick tilt | moves the cursor to the nearest horse or queued marker that way; repeats while held |
| Joystick press | picks the first horse, or drops the cursor |
| Button 1 / 2 | previous / next repo, wrapping |
| Button 3 | jumps to the next horse that needs you, across repos |
| Button 4 | backlight: full, dim, off; while off, any input only wakes it |

With no input for 60 s the cursor goes and the repos rotate every 20 s ([states.md](states.md)). Buttons: read
every 5 ms, a change counts after 4 equal reads, an event on press. Joystick: ADC1 through the chip's calibration
to millivolts; the centre is the mean of 32 reads at boot; a tilt starts past 40 % of the half-range and ends
under 25 %.

**Actions are out of scope.** The LAN listener has no route that changes anything, and stop, kill or answer need a
paired device. The device sends nothing but its `GET`; `tap` fields are ignored; an `arch.toml` ban keeps `POST`
out. Milestone 6 is input for looking, not acting. Acting needs a later decision in drover first.

### 7. The OLED

The status screen, so the TFT stays all scene. Idle: link state and age, the weather line, totals across all repos
(need you, working, queued), and the notes. With a horse selected: its full title, state, age, host and subtitle,
which do not fit the TFT's one-line caption. It comes up first at boot, so it also shows bring-up errors. Its
content shifts by a pixel each minute against burn-in. Dropping the OLED was rejected: the link state would have
to cover the scene.

### 8. The onboard LED

Red while any horse needs you, off otherwise. The blink goes.

### 9. New libraries

Firmware: `embedded-graphics` (text and shapes), `mipidsi` (TFT), `ssd1306` (OLED). Build only: `swag` (the art,
pinned), `resvg` (rasterizer). Tests only: `png` (host screenshots). Wi-Fi, HTTP, ADC and PWM come from
`esp-idf-svc`, already there.

## Consequences

- Every decision above is proved on a laptop: `cargo run --example shot` renders the same frames the board shows
  as PNG files, and CI uploads them. The owner reviews pixels in the PR without a board.
- No design lane is planned: M4's PR carries the rendered pages of every fixture repo. The composition is a few
  constants in `view`, so a revise is cheap.
- The `Ranch` types mirror drover's `scene_screen` by hand. The fixture is the contract test: when drover changes
  the shape, the fixture is re-exported and the test says what broke.
- The Wi-Fi name, password and drover's address are build-time variables; none is in the repo. Anyone holding the
  board can read them from flash, and changing Wi-Fi means a rebuild.
- Anyone on the LAN can read what the device reads; drover#1497 accepts that.
- Moving the swag pin can change the art's palette; `artgen` then fails the build until its slot list matches.

## Out of scope

Actions, touch, a night backdrop (its colours are not in the pack), other themes, OTA updates, on-device Wi-Fi
setup, TLS.

## Follow-on issues

In order; each is one PR. The full text and AC of each is in [milestones.md](milestones.md).

| # | Issue | Size | Needs |
|---|---|---|---|
| M2 | Display bring-up and the app skeleton | M, about +450 lines | – |
| M3 | Model and art pipeline | M, about +500 lines | M2 |
| M4 | Scene render | L, about +550 lines | M3 |
| M5 | Wi-Fi and polling | M, about +350 lines | M4; drover#1497 for its live check only |
| M6 | Input | M, about +350 lines | M5 |

One at a time: each builds on the one before, so no two are in flight together.

## Hot files

- `README.md` would be edited by every milestone. M2 moves its board and build text to `docs/board.md` and
  `docs/build.md`, then it is frozen; each area writes its own `docs/<area>.md`.
- `src/main.rs` is touched by every milestone by nature. It is `shared` and held to 80 lines, so it stays the
  wiring and nothing else.
- `Cargo.toml` is edited by M2 and M3 only; `ci.yml` by M2 only (it tests the whole workspace and uploads
  `target/shots/`, so later milestones add tests without touching it).

## `.drover/arch.toml` lines (the ADR's PR adds all but the `.gpio` ban; M2 adds that one with `pins.rs`)

```toml
frozen = ["README.md"]
shared = ["src/main.rs"]

[[collect]]
dir = "docs"
glob = "*.md"

[comments]
policy = "strict"

[[ban]]
pattern = "esp_idf"
paths = ["src/lib.rs", "src/app.rs", "src/model/**", "src/net/**", "src/input/**", "src/art/**", "src/view/**", "src/render/**", "artgen/**", "examples/**"]
reason = "the library is pure: it builds and tests on the host; esp-idf lives in src/main.rs and src/board"

[[ban]]
pattern = "crate::model"
paths = ["src/render/**"]
reason = "render draws a Frame; view is the only reader of the model"

[[ban]]
pattern = ".gpio"
paths = ["src/**"]
except = ["src/board/pins.rs"]
reason = "the pin map is one file"

[[ban]]
pattern = "Method::Post"
paths = ["src/**"]
reason = "the device is read-only: actions wait for a later decision"

[[size]]
paths = ["src/**/*.rs", "artgen/src/**/*.rs", "examples/*.rs", "build.rs"]
max_file_lines = 300
max_fn_lines = 60

[[size]]
paths = ["src/main.rs"]
max_file_lines = 80
```
