# Board

ESP32-S3 N16R8 (16 MB flash, 8 MB octal PSRAM). The onboard WS2812 is on GPIO48; build with
`--features led-gpio38` for the clones that wire it to GPIO38. The TFT's driver is chosen with a
feature: `ili9341` (the default) or `st7789` (`--no-default-features --features st7789`).

## Pin map

3.3 V everywhere. GPIO 35–37 are PSRAM, 19/20 are USB, and the strapping pins 0, 3, 45 and 46
are avoided. `src/board/pins.rs` is the only file that names a GPIO number.

| Function | GPIO |
|---|---|
| Onboard WS2812 | 48 (38 with `led-gpio38`) |
| TFT SCK / MOSI (shared with touch) | 12 / 11 |
| TFT CS / DC / RST / backlight | 10 / 9 / 14 / 21 |
| Touch T_CS / T_DO / T_IRQ | 15 / 13 / 16 |
| OLED SDA / SCL (SSD1306, I2C) | 17 / 18 |
| Joystick VRx / VRy / SW | 1 / 2 / 4 (ADC1) |
| Buttons 1–4 (pull-up, active low) | 5 / 6 / 7 / 8 |

## Buses

- TFT: SPI2, mode 0, write-only, 20 MHz; landscape 320×240. The touch chip shares the bus and
  its chip-select is held high. The backlight is PWM on GPIO 21.
- OLED: I2C0 at 400 kHz, 128×64.
- Flash: 16 MB with a 4 MB app partition (`partitions.csv`, no OTA). PSRAM: octal, and `malloc`
  falls through to it, so the 150 KB frame buffer lives there.
