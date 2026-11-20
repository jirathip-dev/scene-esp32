use resvg::tiny_skia::{Color, Pixmap, Transform};
use resvg::usvg::{Options, ShapeRendering, Tree};

use crate::{Backdrop, Slot, Sprite};

const MARKERS: [(Slot, [u8; 3]); 5] = [
    (Slot::Coat, [0xff, 0x00, 0xff]),
    (Slot::CoatDark, [0x00, 0xff, 0xff]),
    (Slot::Mane, [0xff, 0xff, 0x00]),
    (Slot::Blaze, [0x00, 0x00, 0xff]),
    (Slot::Fleet, [0x00, 0xff, 0x00]),
];

const SOLID: f32 = 0.2;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Paint {
    Clear,
    Fixed([u8; 3]),
    Slot(Slot),
}

impl Paint {
    pub(crate) fn code(&self) -> String {
        match self {
            Paint::Clear => "Paint::Clear".into(),
            Paint::Fixed(rgb) => format!("Paint::Fixed({})", rgb565(*rgb)),
            Paint::Slot(slot) => format!("Paint::Slot(Slot::{slot:?})"),
        }
    }
}

pub(crate) fn hex(s: &str) -> Option<[u8; 3]> {
    let digits = s.trim().strip_prefix('#').filter(|d| d.len() == 6)?;
    let n = u32::from_str_radix(digits, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

pub(crate) fn rgb565([r, g, b]: [u8; 3]) -> String {
    format!("Rgb565::new({}, {}, {})", r >> 3, g >> 2, b >> 3)
}

fn be565([r, g, b]: [u8; 3]) -> [u8; 2] {
    (u16::from(r >> 3) << 11 | u16::from(g >> 2) << 5 | u16::from(b >> 3)).to_be_bytes()
}

pub(crate) fn backdrop(doc: &str, backdrop: &Backdrop) -> Result<Vec<u8>, String> {
    let doc = vars(doc, doc, &[], "scene.svg")?;
    let hide: String = backdrop
        .hide
        .iter()
        .map(|c| format!(".{c}{{display:none}}"))
        .collect();
    let tree = parse(
        &doc,
        Some(hide),
        ShapeRendering::GeometricPrecision,
        "scene.svg",
    )?;
    let [x, y, w, h] = backdrop.window;
    let (pw, ph) = backdrop.size;
    let mut pixmap = Pixmap::new(pw.into(), ph.into()).ok_or("an empty backdrop")?;
    pixmap.fill(Color::BLACK);
    let scale = Transform::from_scale(f32::from(pw) / w, f32::from(ph) / h).pre_translate(-x, -y);
    resvg::render(&tree, scale, &mut pixmap.as_mut());
    let rgb = pixmap
        .pixels()
        .iter()
        .map(|p| [p.red(), p.green(), p.blue()]);
    Ok(rgb.flat_map(be565).collect())
}

pub(crate) fn sprite(
    sheet: &str,
    sprite: &Sprite,
    slots: &[(&str, Slot)],
) -> Result<(Vec<Vec<u8>>, Vec<Paint>), String> {
    let name = sprite.symbol;
    let open = format!("<symbol id=\"{name}\"");
    let start = sheet
        .find(&open)
        .ok_or_else(|| format!("no symbol {name}"))?;
    let rest = &sheet[start..];
    let head_end = rest
        .find('>')
        .ok_or_else(|| format!("{name}: no end of tag"))?;
    let end = rest
        .find("</symbol>")
        .ok_or_else(|| format!("{name}: no </symbol>"))?;
    let (head, body) = (&rest[..head_end], &rest[head_end + 1..end]);
    let view_box = attr(head, "viewBox").ok_or_else(|| format!("{name}: no viewBox"))?;
    let lower = body.to_ascii_lowercase();
    if let Some((_, rgb)) = MARKERS.iter().find(|(_, c)| lower.contains(&hex_of(*c))) {
        return Err(format!("{name}: uses {}, a slot marker", hex_of(*rgb)));
    }
    let body = solid(&vars(body, sheet, slots, name)?);
    let fixed = fixed_colours(&body);
    let (w, h) = sprite.size;
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="{view_box}">{body}</svg>"#
    );
    let mut palette = vec![Paint::Clear];
    let frames: &[&str] = if sprite.frames.is_empty() {
        &[""]
    } else {
        sprite.frames
    };
    let mut out = Vec::new();
    for frame in frames {
        let others = sprite.frames.iter().filter(|f| *f != frame);
        let sheet: String = others.map(|f| format!(".{f}{{display:none}}")).collect();
        let tree = parse(&svg, Some(sheet), ShapeRendering::CrispEdges, name)?;
        let mut pixmap = Pixmap::new(w.into(), h.into()).ok_or("an empty sprite")?;
        resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
        out.push(indices(&pixmap, &fixed, &mut palette, name)?);
    }
    Ok((out, palette))
}

fn indices(
    pixmap: &Pixmap,
    fixed: &[[u8; 3]],
    palette: &mut Vec<Paint>,
    name: &str,
) -> Result<Vec<u8>, String> {
    let w = pixmap.width() as usize;
    let mut out = Vec::with_capacity(pixmap.pixels().len());
    for (i, p) in pixmap.pixels().iter().enumerate() {
        let c = p.demultiply();
        let rgb = [c.red(), c.green(), c.blue()];
        let paint = classify(rgb, c.alpha(), fixed).ok_or_else(|| {
            let (x, y, a) = (i % w, i / w, c.alpha());
            format!(
                "{name}: pixel ({x}, {y}) is {} at alpha {a}: no fixed colour and no slot",
                hex_of(rgb)
            )
        })?;
        let index = palette.iter().position(|p| *p == paint).unwrap_or_else(|| {
            palette.push(paint);
            palette.len() - 1
        });
        out.push(u8::try_from(index).map_err(|_| format!("{name}: over 256 paints"))?);
    }
    Ok(out)
}

fn classify(rgb: [u8; 3], alpha: u8, fixed: &[[u8; 3]]) -> Option<Paint> {
    let near = |c: &[u8; 3]| rgb.iter().zip(c).all(|(a, b)| a.abs_diff(*b) <= 2);
    if alpha < 128 {
        return Some(Paint::Clear);
    }
    if let Some((slot, _)) = MARKERS.iter().find(|(_, c)| near(c)) {
        return Some(Paint::Slot(*slot));
    }
    fixed.iter().find(|c| near(c)).map(|c| Paint::Fixed(*c))
}

fn fixed_colours(body: &str) -> Vec<[u8; 3]> {
    let markers: Vec<_> = MARKERS.iter().map(|(_, c)| *c).collect();
    let all = body
        .match_indices('#')
        .filter_map(|(i, _)| body.get(i..i + 7).and_then(hex));
    let mut fixed: Vec<_> = all.filter(|c| !markers.contains(c)).collect();
    fixed.sort_unstable();
    fixed.dedup();
    fixed
}

fn solid(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find("opacity=\"") {
        let (head, tail) = rest.split_at(at + 9);
        let (value, tail) = tail.split_once('"').unwrap_or((tail, ""));
        let on = value.parse::<f32>().map_or(true, |v| v >= SOLID);
        out.push_str(head);
        out.push_str(if on { "1\"" } else { "0\"" });
        rest = tail;
    }
    out.push_str(rest);
    out
}

fn hex_of([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    tag[at..].split('"').next()
}

fn vars(text: &str, doc: &str, slots: &[(&str, Slot)], symbol: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("var(") {
        out.push_str(&rest[..at]);
        let inner = &rest[at + 4..];
        let close = inner
            .find(')')
            .ok_or_else(|| format!("{symbol}: an unclosed var("))?;
        let (name, fallback) = match inner[..close].split_once(',') {
            Some((n, f)) => (n.trim(), Some(f.trim())),
            None => (inner[..close].trim(), None),
        };
        let slot = slots.iter().find(|(n, _)| *n == name).map(|(_, s)| *s);
        let marker = slot
            .and_then(|s| MARKERS.iter().find(|(m, _)| *m == s))
            .map(|(_, c)| hex_of(*c));
        let value = marker
            .or(fallback.map(str::to_owned))
            .or_else(|| declared(doc, name).map(str::to_owned))
            .ok_or_else(|| format!("{symbol}: var({name}) has no fallback and no slot"))?;
        out.push_str(&value);
        rest = &inner[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn declared<'a>(doc: &'a str, name: &str) -> Option<&'a str> {
    let decl = format!("{name}:");
    doc.match_indices(&decl).find_map(|(i, _)| {
        let before = doc[..i].chars().next_back()?;
        if before.is_ascii_alphanumeric() || before == '-' {
            return None;
        }
        doc[i + decl.len()..]
            .split([';', '}', '"'])
            .next()
            .map(str::trim)
    })
}

fn parse(
    svg: &str,
    style_sheet: Option<String>,
    shape: ShapeRendering,
    what: &str,
) -> Result<Tree, String> {
    let options = Options {
        shape_rendering: shape,
        style_sheet,
        ..Options::default()
    };
    Tree::from_str(svg, &options).map_err(|e| format!("{what}: {e}"))
}
