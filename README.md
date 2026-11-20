# scene-esp32

ESP32-S3 companion device: a Blocks renderer of drover's Scene.

It fetches `/screens/scene` from drover's read-only LAN listener and draws it with the
[blocks](https://github.com/jirathip-dev/blocks) types. The firmware never imports drover code,
only its served JSON; `blocks` is a git dependency pinned to one full commit.

## Board

ESP32-S3 N16R8 (16 MB flash, 8 MB octal PSRAM). The onboard WS2812 is on GPIO48; build with
`--features led-gpio38` for the clones that wire it to GPIO38. The display driver is chosen with
`--features ili9341` or `--features st7789` (declared, not wired up yet).

## Build and flash

```sh
cargo install espup espflash ldproxy
espup install
. ~/export-esp.sh
cargo run --release
```

`cargo run` flashes with `espflash flash --monitor`: the LED blinks and the log shows the
fixture's block count. The host-side test of the fixture parse:

```sh
cargo test --lib --target x86_64-unknown-linux-gnu
```

(use your host's triple, e.g. `aarch64-apple-darwin`).

## Pin map

3.3 V everywhere. GPIO 35–37 are PSRAM, 19/20 are USB, and the strapping pins 0, 3, 45 and 46
are avoided.

| Function | GPIO |
|---|---|
| Onboard WS2812 | 48 (38 with `led-gpio38`) |
| TFT SCK / MOSI (shared with touch) | 12 / 11 |
| TFT CS / DC / RST / backlight | 10 / 9 / 14 / 21 |
| Touch T_CS / T_DO / T_IRQ | 15 / 13 / 16 |
| OLED SDA / SCL (SSD1306, I2C) | 17 / 18 |
| Joystick VRx / VRy / SW | 1 / 2 / 4 (ADC1) |
| Buttons 1–4 (pull-up, active low) | 5 / 6 / 7 / 8 |

## License

MIT
