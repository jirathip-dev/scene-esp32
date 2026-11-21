# ADR: the two hot files (#15)

## Summary

Decision: The one screenshot script becomes a small host-only crate where each feature adds its own file; the
normal test run writes the same 15 pictures, so CI loses a step. The library's module list is frozen at its seven
modules: new code goes inside a module's folder, never into the list.
Slices: one PR in two commits: the screenshot crate (the old script deleted), then the freeze (one dead constant
removed).
Open questions: none.

## The question

Five of the last eight merged PRs edited `examples/shot.rs` and `src/lib.rs`. Both are central lists:

- `examples/shot.rs` (166 lines) lists every screenshot. Each milestone added its pictures to the one `main`, and
  two of them rewrote another milestone's lines to do it.
- `src/lib.rs` (26 lines) lists the library's modules. Each milestone added its `pub mod` line.

What layout lets the next features add a screenshot or code without editing a file another PR edits?

## Checked by a run

Scratch crates and a scratch git repo, cargo 1.97.0-nightly and drover `48f08b4` (CI pins `build-staging-9ea69d9`).

| Question | Run | Result |
|---|---|---|
| Can screenshots be tests in the root `tests/`? | `cargo test --workspace` with one root integration test and a binary that needs esp-idf | exit 101: Cargo builds the firmware binary for any root integration test: `unresolved import esp_idf_svc` |
| Can they be tests of a member crate? | the same, tests in `shots/tests/`, the kit in `shots/src/lib.rs` | exit 0; both PNG files written; the binary not built |
| Can a module list be generated or included? | `cargo fmt --check` on a crate whose `mod` line sits behind `include!`, with a misformatted module | exit 0: rustfmt never opens the module (exit 1 with a plain `mod` line) |
| Does the freeze stop the PR that adds it? | `drover arch-check --base HEAD^1` on a commit that freezes `src/lib.rs` and adds a line to it | exit 0: the freeze is read from the base |
| Can one PR lift the freeze and add a line? | the same, the commit drops `src/lib.rs` from `frozen` and adds `pub mod ota;` | exit 1, for the same reason |
| Does `frozen` stop a deleted file's return? | the same, a later commit re-creates `examples/shot.rs` | exit 0: it does not; a `[[ban]]` on `fn main` in `examples/**` does (exit 1) |

## Options

### `examples/shot.rs`

| | Option | Verdict |
|---|---|---|
| A | One example in a folder, a file per feature, `main.rs` naming each | `main.rs` is the same list, shorter. Listing it from `build.rs` hides the files from rustfmt (row 3) |
| B | One example per feature, `examples/shot_*.rs`, found by Cargo | No single command: CI and the docs need a shell loop. The shared helpers become a `mod` each file repeats |
| C | Integration tests in the root `tests/` | Does not build on the host (row 1) |
| D | Data scripts run by one fixed runner | A script format and its parser for 15 pictures; the storm and the art sheet do not fit it |
| E | PNG writes inside each module's unit tests | A `view` or `net` test would have to call `render`, against the firmware ADR's module table |
| **F** | **A host-only workspace crate `shots`: the kit in its library, one test file per feature in `shots/tests/`** | **Chosen** |

### `src/lib.rs`

| | Option | Verdict |
|---|---|---|
| **A** | **Freeze it at the seven modules; new code is a file in a module's folder** | **Chosen** |
| B | Generate the list (`build.rs`, or a macro crate that reads the folder) | rustfmt and the editor find files through `mod` lines in the source: `cargo fmt --check` would pass an unformatted library (row 3). The macro crate is also a new dependency |
| C | Mark it `shared` under a line limit, as `src/main.rs` is | `shared` is for code every feature must touch. No feature needs this file any more; only an eighth module does |

## Decision

**Screenshots: option F.** `examples/shot.rs` is deleted. Its helpers move to `shots/src/lib.rs` and its `main`
splits by feature into `shots/tests/{scene,link,browse,art}.rs`. Cargo's own test discovery is the glob: a new
`shots/tests/<feature>.rs` is found with no line anywhere else. The shapes and a whole file are in
[README.md](README.md).

**Module list: option A.** The seven modules are the firmware ADR's architecture, and that list is complete: the
five milestones that each added a line are merged. `src/lib.rs` is frozen. A feature adds a file under
`src/<module>/` and its module's own `mod` line, which is that area's list and no other area's.

## Consequences

- A feature with something to show adds `shots/tests/<feature>.rs` and its `docs/<area>.md` line; it edits no
  shared file. Two PRs conflict on a screenshot only when they change the same feature's file.
- The command changes: `cargo run --example shot` becomes `cargo test -p shots`, and `cargo test --workspace`
  (CI's existing line) writes the pictures too. CI's `cargo run --example shot` step goes; the upload step, which
  fails on an empty folder, stays as it is.
- A screenshot that cannot be drawn now fails the test run by name, instead of stopping the script at the first
  error.
- The library loses its only dev-dependency; `png` belongs to `shots`. The firmware build never builds `shots`:
  `cargo build` at the root builds the root package only, as it already does with `artgen`.
- An eighth library module is an architecture change and costs one small PR first: a one-line PR takes
  `src/lib.rs` out of `frozen`, then the module's PR adds its line and puts the freeze back (rows 4 and 5). That
  is the price of the freeze, paid only then.
- The firmware ADR's pages still say `examples/shot.rs`. They stay as the record of that decision; this ADR
  replaces those lines.
- `frozen` alone would let `examples/shot.rs` come back (row 6), so a ban on `fn main` under `examples/**` keeps
  one way to take a screenshot. A future example that is not a screenshot needs that ban's `except`.

## AC for the impl PR

- [ ] `cargo test --workspace --target <host>` passes and writes the same 15 PNG files to `target/shots/`, each
      byte-identical to the file `cargo run --example shot` wrote at the base commit. The PR body shows the
      compare command and its exit code.
- [ ] `examples/` is gone and the root `Cargo.toml` has no `[dev-dependencies]`; `Cargo.lock` gains only the
      `shots` package.
- [ ] CI's `host` job has no `cargo run --example` step, and its `shots` artifact holds the 15 files.
- [ ] `PANEL` is gone from `src/lib.rs`, and both firmware builds still pass.
- [ ] `docs/build.md`, `docs/scene.md` and `docs/art.md` name the new command and the files that exist
      (`docs/build.md` still names `boot.png`, gone since milestone 4).
- [ ] `drover arch-check` passes with the lines below.

## `.drover/arch.toml` lines (the impl PR adds them in the same PR)

| Old list | Frozen | Collected folder | Collected by |
|---|---|---|---|
| `examples/shot.rs` (deleted) | yes | `shots/tests/*.rs` | Cargo's test discovery |
| `src/lib.rs` (kept) | yes | `src/*/*.rs` | each module's own `mod` lines |

The first line replaces `frozen = ["README.md"]`:

```toml
frozen = ["README.md", "src/lib.rs", "examples/shot.rs"]
```

New tables, after the `docs` one:

```toml
[[collect]]
dir = "shots/tests"
glob = "*.rs"

[[collect]]
dir = "src"
glob = "*/*.rs"

[[ban]]
pattern = "fn main"
paths = ["examples/**"]
reason = "screenshots are tests: add a file in shots/tests"
```

Two edits to lines that exist: in the `esp_idf` ban's `paths`, `"examples/**"` becomes `"shots/**"`; in the first
`[[size]]` rule's `paths`, `"examples/*.rs"` becomes `"shots/**/*.rs"`.
