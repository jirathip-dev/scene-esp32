# scene-esp32

ESP32-S3 companion device: a Blocks renderer of drover's Scene.

It fetches `/screens/scene` from drover's read-only LAN listener and draws it with the
[blocks](https://github.com/jirathip-dev/blocks) types. The firmware never imports drover code,
only its served JSON; `blocks` is a git dependency pinned to one full commit.

The board, its pin map and buses are in [docs/board.md](docs/board.md); building, flashing and
the host tests are in [docs/build.md](docs/build.md).

## License

MIT
