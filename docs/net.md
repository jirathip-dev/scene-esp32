# Wi-Fi and polling

The board joins Wi-Fi and polls drover's `GET http://<SCENE_ADDR>/screens/scene` every 5 s.

## What is compiled in

Three values are read from the build's environment and compiled into the firmware:

| Variable | What |
|---|---|
| `SCENE_WIFI_SSID` | the Wi-Fi network's name |
| `SCENE_WIFI_PASS` | its password; unset or empty joins an open network |
| `SCENE_ADDR` | drover's address, an IPv4 address and port such as `192.168.1.20:7779` |

```sh
export SCENE_WIFI_SSID=… SCENE_WIFI_PASS=… SCENE_ADDR=192.168.1.20:7779
cargo run --release
```

A `SCENE_ADDR` that is not an IPv4 address and port fails the build. With the name or the address unset (as in CI)
the firmware builds, polls nothing and shows "no drover: not configured".

**They are plain text in the flash.** Anyone holding the board can dump the flash and read the Wi-Fi name, its
password and drover's address. Use a network whose password you are fine to leave on the device, and reflash
with other values to change them.

## Polling

- A request times out after 4 s. A body is read up to 256 KB; one that fills it is `too big`.
- A body equal to the last good one is not parsed again and redraws nothing. One that does not parse is
  `bad data` and leaves the last scene on screen.
- After a good poll the next one is 5 s later. After failures in a row the waits are 5, 10, 20, 40, then 60 s.
  Joining Wi-Fi is part of the poll and backs off the same way.

## What the screens show

| Last good poll | TFT | OLED, first line |
|---|---|---|
| none yet, under 20 s | the loading horse, "saddling up" | `joining wifi` |
| up to 20 s ago | the scene | `fresh 3s` |
| up to 5 min ago | the scene, an amber `stale 45s` in the header | `stale 45s: timeout` |
| older, or none in 20 s | the loading horse, "no drover" and the reason | `no drover: no wifi` |

The reasons: `not configured`, `no wifi`, `refused`, `timeout`, `http 403`, `http 404`, `too big`, `bad data`.
The next good poll brings the scene back.
