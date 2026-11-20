mod svg;

use std::fmt::Write as _;
use std::path::Path;

/// What the build takes from a pack: the backdrop's window, the slot variables and the sprites.
pub struct Theme {
    pub backdrop: Backdrop,
    pub slots: &'static [(&'static str, Slot)],
    pub sprites: &'static [Sprite],
}

/// The part of `scene.svg` drawn behind the scene, `[x, y, width, height]` in its units, and the classes hidden.
pub struct Backdrop {
    pub window: [f32; 4],
    pub size: (u16, u16),
    pub hide: &'static [&'static str],
}

/// One symbol of `sprites.svg`, its size in pixels and its frames by class (none: the whole symbol).
pub struct Sprite {
    pub name: &'static str,
    pub symbol: &'static str,
    pub size: (u16, u16),
    pub frames: &'static [&'static str],
}

/// A colour the device fills in as it draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot {
    Coat,
    CoatDark,
    Mane,
    Blaze,
    Fleet,
}

/// The art's limit in flash.
pub const MAX_BYTES: usize = 256 * 1024;

/// Rasterizes `pack` by `theme` into `out`: the pixel files and `art.rs`, which includes them.
pub fn build(theme: &Theme, pack: &swag::Pack, out: &Path) -> Result<(), String> {
    let mut code = chrome(pack)?;
    let (w, h) = theme.backdrop.size;
    let pixels = svg::backdrop(text(pack, "scene.svg")?, &theme.backdrop)?;
    let mut bytes = pixels.len();
    let path = write(out, "backdrop.bin", &pixels)?;
    let _ = writeln!(code, "pub static BACKDROP: Backdrop = Backdrop {{ size: ({w}, {h}), pixels: include_bytes!({path:?}) }};");

    let sheet = text(pack, "sprites.svg")?;
    for sprite in theme.sprites {
        let (frames, palette) = svg::sprite(sheet, sprite, theme.slots)?;
        let mut includes = Vec::new();
        for (i, frame) in frames.iter().enumerate() {
            bytes += frame.len();
            let path = write(out, &format!("{}-{i}.bin", sprite.name), frame)?;
            includes.push(format!("include_bytes!({path:?})"));
        }
        let palette: Vec<_> = palette.iter().map(svg::Paint::code).collect();
        let (w, h) = sprite.size;
        let _ = writeln!(
            code,
            "static {}: Sprite = Sprite {{ size: ({w}, {h}), frames: &[{}], palette: &[{}] }};",
            sprite.name,
            includes.join(", "),
            palette.join(", ")
        );
    }
    if bytes >= MAX_BYTES {
        return Err(format!("the art is {bytes} bytes, over {MAX_BYTES}"));
    }
    write(out, "art.rs", code.as_bytes()).map(drop)
}

fn text<'a>(pack: &'a swag::Pack, name: &str) -> Result<&'a str, String> {
    let (_, bytes) = pack
        .files
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| format!("pack {} has no {name}", pack.name))?;
    std::str::from_utf8(bytes).map_err(|e| format!("{name}: {e}"))
}

fn write(out: &Path, name: &str, bytes: &[u8]) -> Result<String, String> {
    let path = out.join(name);
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

fn chrome(pack: &swag::Pack) -> Result<String, String> {
    let theme: serde_json::Value =
        serde_json::from_str(text(pack, "theme.json")?).map_err(|e| format!("theme.json: {e}"))?;
    let name = theme["scheme"]["dark"]
        .as_str()
        .ok_or("theme.json names no dark scheme")?;
    let (_, json) = swag::schemes()
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| format!("swag has no scheme {name}"))?;
    let scheme: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("{name}: {e}"))?;
    let token = |t: &str| {
        let hex = scheme["tokens"][t]
            .as_str()
            .ok_or(format!("scheme {name} has no {t}"))?;
        svg::hex(hex)
            .map(svg::rgb565)
            .ok_or(format!("scheme {name}: {t} is {hex}"))
    };
    let fleet = ["--fk-l", "--fk-p", "--fk-c", "--fk-g", "--fk-a"].map(token);
    let fleet: Result<Vec<_>, String> = fleet.into_iter().collect();
    Ok(format!(
        "pub static CHROME: Chrome = Chrome {{ bg: {}, fg: {}, mute: {}, good: {}, warn: {}, bad: {}, fleet: [{}] }};\n",
        token("--bg")?,
        token("--fg")?,
        token("--mute")?,
        token("--g")?,
        token("--a")?,
        token("--r")?,
        fleet?.join(", ")
    ))
}
