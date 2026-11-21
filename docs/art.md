# Art

The build turns swag's `farm` pack, pinned by commit in `Cargo.toml`, into pixels; the chip never reads SVG.
`build.rs` names what is taken (the `FARM` theme) and `artgen` rasterizes it with `resvg` into the build folder.
Nothing generated is committed.

| What | From | Out |
|---|---|---|
| Backdrop | `scene.svg`, units 66.5 to 320 down, day only | 320×208, raw big-endian RGB565 |
| Seven horses | `s-scene-edit`, `-idle`, `-done`, `-needs`, `-fail`, `s-loading`, `s-empty` | 66×50, one palette index a pixel |
| The bale | `s-needs`, frames `kf1` and `kf2` | 32×24, two frames |
| Chrome | the pack's dark scheme: `--bg`, `--fg`, `--mute`, `--g`, `--a`, `--r`, `--fk-*` | `art::CHROME` |

A sprite's palette entry is clear, a fixed colour, or a slot (`--c` coat, `--d` coat dark, `--m` mane, `--b`
blaze, `--fk` fleet) filled from the row's coat and the repo's colour as it draws.

How a `var()` is read: a slot first, then its fallback, then the value the file declares for it (the backdrop's
`#night{…}` day colours). Anything else fails the build with the symbol's name.

Sprites are drawn without anti-aliasing, and each element's opacity is rounded to on (0.2 and over) or off, so
every pixel is exactly one paint. A pixel that is no fixed colour of its symbol and no slot fails the build with
the symbol and the pixel. So does art of 256 KB or more.

`cargo test -p shots --test art --target <host>` (`shots/tests/art.rs`) writes `target/shots/art.png`: the backdrop, then every sprite in the
pack's default coat and in two of the fixture's coats.
