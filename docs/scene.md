# Scene

`view` turns a snapshot into a display list; `render` draws that list and nothing else. The layout and the
state-to-sprite table are in [architecture/firmware/shapes.md](architecture/firmware/shapes.md), section 3.

| What | Where (screen pixels) |
|---|---|
| Header: fleet tick, repo, its counts, page dots | rows 0–15 |
| Queued markers (the red bale when it needs you), up to 12, then `+N` | y 200, x 4 + 24·i |
| Horses: front row slots 0–4, back row slots 5–9, then `+N` | front y 146, x 29 + 56·i; back y 96, x 1 + 56·i |
| Caption: the selected row, else the weather line | rows 224–239 |

- Order of horses: need you (blocked, dead), working, idle, done; drover's order inside each.
- A working horse sways 4 to 12 px with the wind (one pixel a 100 ms tick) and bobs one pixel; the loop then
  ticks every 100 ms, else every second. The bale swaps its two frames each second.
- Fog: `fog` 1 (or the `fog` sky) is a light veil, 2 and over a thick one. Rain is drawn when `storm` is set
  or the sky is `storm`.
- Text is cut in the middle with `~` (Latin-1 has no ellipsis); a character outside Latin-1 becomes `?`.
- Ambient starts on the first repo with a row that needs you and turns to the next repo with rows every 20 s.
- `render::draw` compares the new list with the last one by item key and repaints only the rectangles that
  changed, from the backdrop up; the TFT flush sends just those rectangles and logs its time.
- The OLED: the link and its age, the weather line, totals across repos, then the notes; with a row selected,
  its full title, state and age, host and subtitle. It shifts a pixel each minute.

`cargo run --example shot --target <host>` writes `scene-<repo>.png` for each fixture repo, `loading`,
`storm-fog`, `browse`, `oled-idle` and `oled-horse` to `target/shots/`. `GOLDEN=1 cargo test --target <host>`
rewrites `tests/golden/<repo>.txt`, the display list of each fixture repo.
