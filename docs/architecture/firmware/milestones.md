# Follow-on issues: milestones 2–6 (#2)

In order. Each is one PR and each ends with something to look at: PNG files from `cargo run --example shot` in
CI, and a serial log from the board. A board line in an AC is run by whoever has the board on USB (the owner, or a
lane on that host); the PR body says who ran it and pastes the log.

Sizes are `src/` plus `artgen/` net lines, tests included: S under 200, M 200–500, L over 500.

| # | Issue | Size | Needs | Owns (no other milestone edits these) |
|---|---|---|---|---|
| M2 | Display bring-up and the app skeleton | M, about +450 | – | `src/board/{mod,pins,tft,oled,led}.rs`, `src/render/buf.rs`, `examples/shot.rs`, `sdkconfig.defaults`, `partitions.csv`, `ci.yml`, `.drover/arch.toml`, `docs/board.md`, `docs/build.md` |
| M3 | Model and art pipeline | M, about +500 | M2 | `src/model/`, `src/art/`, `artgen/`, `build.rs`, `docs/art.md` |
| M4 | Scene render | L, about +550 | M3 | `src/view/`, `src/render/{scene,status}.rs`, `tests/golden/`, `docs/scene.md` |
| M5 | Wi-Fi and polling | M, about +350 | M4 | `src/net/`, `src/board/wifi.rs`, `docs/net.md` |
| M6 | Input | M, about +350 | M5 | `src/input/`, `src/board/controls.rs`, `docs/input.md` |

One at a time, in this order. `src/app.rs` and `src/main.rs` are shared: M2 writes both and each later milestone
adds its part. `Cargo.toml` changes in M2 (firmware libraries) and M3 (the workspace and build libraries) only.

---

## M2. Display bring-up and the app skeleton

**Problem.** Nothing draws yet, and the two panels, the SPI speed and PSRAM are the hardware risks. They are
settled first, with the loop every later milestone plugs into.

**Wanted.**

- `src/board/`: `pins.rs` (the README's pin map as named parts, the only file with GPIO numbers), `tft.rs`
  (`mipidsi`; the `ili9341` / `st7789` features pick model, colour order and inversion; landscape; SPI2 at 20 MHz;
  touch chip-select held high; backlight PWM), `oled.rs` (`ssd1306`, I2C at 400 kHz), `led.rs` (the WS2812 code
  moved out of `main`).
- `sdkconfig.defaults` and `partitions.csv`: octal PSRAM with `malloc` falling through to it, 16 MB flash, a 4 MB
  app partition.
- The library's skeleton: `render::FrameBuf` (320×240 in PSRAM, tracks the rows touched), `App`, `Event`, a
  boot `Frame` (three colour bars with their names, the panel's name, the uptime) and a boot `Status`.
- `main.rs` as in [usage.md](usage.md), without the two threads. `examples/shot.rs`. CI: both panel builds, the
  workspace's host tests, `shot` with its PNGs uploaded, arch-check.
- The `.gpio` ban from the ADR in `.drover/arch.toml`. The README's board and build text moves to `docs/`, and its "not wired up
  yet" line goes. The README only loses and edits lines, so the freeze passes in the same PR.

**AC.**

- [ ] `cargo build --release` and `cargo build --release --no-default-features --features st7789` build in CI;
      both features, or neither, fail with one message that names the two features.
- [ ] `cargo run --example shot --target <host>` writes `target/shots/boot.png` (320×240) and `oled-boot.png`
      (128×64); CI uploads them.
- [ ] A host test: drawing the same frame twice touches no row the second time.
- [ ] Board: the log shows 8 MB of PSRAM and the frame buffer's address inside it; the TFT shows the bars in
      landscape with red, green and blue under their own names; the OLED shows the boot status.
- [ ] Board: 10 minutes of full-frame redraws at 20 MHz with no wrong pixel; the log gives the frame time
      (expected near 61 ms).
- [ ] `drover arch-check` passes: no esp-idf in the library, no GPIO number outside `pins.rs`.

## M3. Model and art pipeline

**Problem.** The scene's data and art have no types yet: `block_count` only proves that the fixture parses.

**Wanted.**

- `src/model/`: `parse` and the types of [shapes.md](shapes.md) section 2. `block_count` goes, and its caller
  logs the snapshot's repo and row counts instead.
- `artgen/` (a workspace crate for the host only), `swag` pinned by full commit as a build-dependency, and
  `build.rs` as in [usage.md](usage.md): the backdrop, the eight sprites (seven horses and the bale), the chrome
  colours.
- `src/art/`: the types of shapes.md section 4 over the built files.
- `shot` also writes `art.png`: the backdrop, and every sprite in the pack's default coat and in two of the
  fixture's coats.

**AC.**

- [ ] `model::parse` on the fixture gives 5 repos and 19 rows, one note and nothing not shown; the row
      `h-acme-api-impl-17` is `Working` with coat `#b08d5e`.
- [ ] Tests: a body that is not a screen, a screen with no scene, and a scene whose `repos` is a number each give
      their own error; an unknown `state` or `sky` word gives `Other`; an unknown field is ignored.
- [ ] The build writes a 320×208 backdrop and the eight sprites from swag at the pinned commit, and fails,
      naming the symbol, on a pixel that is no fixed colour and no slot, or a `var()` with no fallback and no
      slot (tested with a broken SVG).
- [ ] The art is under 256 KB (the build asserts it) and the firmware's dependency tree has no SVG code (a CI
      line checks that `cargo tree -e normal` names no `resvg` or `usvg`).
- [ ] The PR body gives the measured peak memory of parsing the fixture, and the same for a 256 KB scene.

## M4. Scene render

**Problem.** The device can hold a scene and its art but draws neither.

**Wanted.**

- `src/view/`: `frame` and `status` as in shapes.md section 3: the page layout, the state-to-sprite table,
  texts cut to fit, totals, the Ambient rotation.
- `src/render/`: `draw` (backdrop, markers, sprites with their slots filled, fog veil, rain, header, caption,
  cursor) and `status` for the OLED. A changed item redraws only its own rectangle, restored from the backdrop.
- Until M5 the board shows the fixture: a boot thread with a 32 KB stack parses it and sends it as the one poll.
- The LED is red while any row needs you.

**AC.**

- [ ] `shot` writes one PNG per fixture repo (5), plus loading, a storm with fog, and both OLED pages; the PR
      body shows them.
- [ ] Each fixture repo's display list matches `tests/golden/<repo>.txt`.
- [ ] A test with 14 lanes in one repo: 10 horses drawn and a `+4`; none dropped without a count.
- [ ] A pixel test: the horse of `h-acme-api-impl-17` has coat pixels of `#b08d5e` and blanket pixels of the
      repo's fleet colour.
- [ ] A test: one animation tick with one working horse touches at most that horse's two rectangles.
- [ ] A title of 60 characters is cut in the middle to fit; a character outside Latin-1 becomes `?`.
- [ ] Board: the fixture's repos rotate every 20 s; a working horse walks; the log shows a tick's flush under
      10 ms.

## M5. Wi-Fi and polling

**Problem.** The scene on the board is the fixture, not drover's.

**Wanted.**

- `src/net/`: `Feed` and `Link` as in shapes.md section 5 and [states.md](states.md) section 1.
- `src/board/wifi.rs`: join, `GET` with the 4 s timeout and the 256 KB cap, the net thread.
- `build.rs` reads `SCENE_WIFI_SSID`, `SCENE_WIFI_PASS` and `SCENE_ADDR`.
- The stale and lost looks on both screens. The fixture and M4's boot thread leave the firmware.

**AC.**

- [ ] Tests, with scripted polls and times: fresh up to 20 s, stale up to 5 min, then lost, and fresh again on
      the next good poll; waits of 5, 10, 20, 40, 60, 60 s after failures in a row and 5 s after a good poll.
- [ ] Tests: the same body twice is parsed once and redraws nothing; a body over 256 KB is `TooBig` and no more
      than 256 KB of it was read; a body that does not parse leaves the last scene on screen.
- [ ] `SCENE_ADDR=example.com:80` fails the build; with the three unset the firmware builds and shows
      "not configured".
- [ ] `shot` writes `stale.png`, `lost.png` and their OLED pages.
- [ ] Board, against a host serving `fixtures/scene.json` at `/screens/scene`: the scene shows within 10 s of
      power-on; with the server stopped the header says stale within 25 s and the screen says "no drover" at
      5 min; it is back within 65 s of the server's return. The log is in the PR body.
- [ ] Board, against drover with `device_addr` set (after drover#1497): the live scene shows. This line alone
      waits for that issue.
- [ ] `docs/net.md` says what is compiled in (Wi-Fi name and password, the address) and that it can be read off
      the board.

## M6. Input

**Problem.** The scene can only be watched: no repo can be chosen and no horse read in full.

**Wanted.**

- `src/input/`: `Debounce` and `Stick` as in states.md section 3.
- `src/board/controls.rs`: the four buttons and the stick's switch with pull-ups, ADC1 with the chip's
  calibration, the input thread at 5 ms.
- `app`: `Mode` and the backlight as in states.md section 2; the cursor, the caption and the OLED's horse page.
- No action is sent: this milestone is input for looking. Acting on a lane waits for a later decision.

**AC.**

- [ ] Tests on read sequences: a 5 ms glitch sends nothing; a 30 ms press sends one `Press`; a release sends
      nothing.
- [ ] Tests: a centre off by 5 % sends nothing; a tilt past 40 % sends one `Tilt`, then one at 400 ms and every
      150 ms; back under 25 % it stops; with both axes tilted the larger wins.
- [ ] Tests on `App` with the fixture: a tilt picks the nearest item that way and stays when there is none;
      buttons 1 and 2 wrap; button 3 visits every row that needs you, in drover's order, across repos; the
      cursor stays on its row when a new snapshot moves it, and drops when the row is gone; 60 s without input
      returns to Ambient.
- [ ] Tests: button 4 goes full, dim, off; while off an input only wakes; a rise in the count that needs you
      wakes.
- [ ] `shot` writes `browse.png` (cursor and caption) and `oled-horse.png`.
- [ ] `drover arch-check` passes with the ban on `Method::Post`, and `docs/input.md` lists the controls.
- [ ] Board: each control does what the ADR's table says; the log shows the learned centre and one line per
      event.
