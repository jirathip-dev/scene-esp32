# Input

The controls are for looking only: the device sends nothing but its `GET`, and an `arch.toml` ban keeps
`Method::Post` out of `src/`. The states are in [architecture/firmware/states.md](architecture/firmware/states.md),
sections 2 and 3.

| Control | Pin | What it does |
|---|---|---|
| Joystick tilt | VRx 1, VRy 2 (ADC1) | moves the cursor to the nearest horse or queued marker that way; stays when there is none; repeats while held |
| Joystick press | SW 4 | picks the first horse, or drops the cursor and goes back to Ambient |
| Button 1 | 5 | previous repo, wrapping |
| Button 2 | 6 | next repo, wrapping |
| Button 3 | 7 | the next row that needs you, in drover's order across repos, wrapping; with none the caption says so |
| Button 4 | 8 | backlight: full, dim, off; while off, any input only wakes it |

- With a cursor the caption shows the row and the OLED shows it in full: title, state and age, host, subtitle.
- 60 s with no input drops the cursor and returns to Ambient, which turns to the next repo every 20 s.
- Off darkens the backlight and the OLED; polling and the LED go on. A rise in the count that needs you wakes it.
- Buttons and the stick's switch use the chip's pull-ups (active low) and are read every 5 ms; a change counts
  after 4 equal reads, and only the press sends.
- The stick is read through ADC1 at 12 dB with the chip's curve calibration, in millivolts (full scale
  3,100 mV). Its centre is the mean of 32 reads at boot, logged as `stick centre [x, y] mV`; an axis whose mean
  is outside 35–65 % of full scale (the stick was held) uses the middle. A tilt starts past 40 % of the
  half-range, repeats at 400 ms and then every 150 ms, and ends under 25 %. With both axes tilted the larger wins.
  A higher reading on x is right and on y is down.
- Each input is logged as one line, `input Press(B3)` or `input Tilt(Left)`.
