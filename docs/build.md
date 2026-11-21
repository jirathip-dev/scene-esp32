# Build and flash

```sh
cargo install espup espflash ldproxy
espup install
. ~/export-esp.sh
cargo run --release
cargo run --release --no-default-features --features st7789
```

`cargo run` flashes with `espflash flash --monitor` and the partition table in `partitions.csv`.
At boot the TFT shows red, green and blue bars under their names with the panel's name and the
uptime; the OLED shows the boot status. The log gives the PSRAM size, the frame buffer's address
and the time of a full-frame flush.

`--features soak` resends the whole frame every second, for the 20 MHz soak test.

## On the host

The library and its tests run on the host (use your host's triple, e.g. `aarch64-apple-darwin`):

```sh
cargo test --workspace --target x86_64-unknown-linux-gnu
```

The `shots` crate's tests (`cargo test -p shots`, one file per feature in `shots/tests/`) write the 15
screenshots to `target/shots/`: TFT frames at 320×240, `oled-*` at 128×64 and the `art` sheet. CI uploads
that folder.
