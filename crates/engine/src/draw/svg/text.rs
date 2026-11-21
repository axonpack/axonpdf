//! SVG `Text` and `Tspan`: one line per span, placed at `x`/`y` (spans without `x` follow the
//! previous one), anchored by `textAnchor` and shifted by `dominantBaseline`.

use serde_json::{Map, Value};

use super::props::Props;
use crate::display::{Color, FontId, Op, TextRun};
use crate::fonts::{FontStore, weight_from_name};
use crate::tree::{Kind, Node};

pub fn draw(ops: &mut Vec<Op>, text: &Node, p: &Props, fonts: &FontStore) {
    let inherited = p.inherited(true);
    // Loose text is a span of its own.
    let spans: Vec<(Props, String)> = text
        .children
        .iter()
        .map(|c| match c.kind {
            Kind::TextInstance => (
                Props::of(
                    &Node {
                        props: Map::new(),
                        ..c.clone()
                    },
                    &inherited,
                    p.container,
                ),
                c.value.clone(),
            ),
            _ => (
                Props::of(c, &inherited, p.container),
                c.children.iter().map(|t| t.value.as_str()).collect(),
            ),
        })
        .collect();
    let mut pen = p.n("x");
    let count = spans.len();
    for (i, (sp, mut s)) in spans.into_iter().enumerate() {
        // Spans that follow on are joined with a space.
        if sp.get("x").is_none() && i + 1 < count && !s.is_empty() {
            s.push(' ');
        }
        let s = transform(&s, sp.str("textTransform"));
        let font = font(&sp, fonts);
        let size = sp.num("fontSize").unwrap_or(18.0) as f32;
        let spacing = sp.n("letterSpacing") as f32;
        let glyphs = fonts.shape(font, &s, size, spacing, &[]);
        let width: f32 = glyphs.iter().map(|g| g.x_advance).sum();
        let x = sp.num("x").unwrap_or(pen) as f32;
        let y = sp.n("y") as f32;
        pen = (x + width) as f64;
        let x = match sp.str("textAnchor") {
            Some("middle") => x - width / 2.0,
            Some("end") => x - width,
            _ => x,
        };
        let m = fonts.metrics(font);
        let scale = size / m.units_per_em;
        let y = y + scale
            * match sp.str("dominantBaseline") {
                Some("middle" | "central") => m.cap_height / 2.0,
                Some("hanging") => m.cap_height,
                Some("mathematical") => m.x_height,
                Some("text-after-edge") => m.descent,
                Some("text-before-edge") => m.ascent,
                _ => 0.0,
            };
        let color = sp.color("fill").unwrap_or(Color::BLACK);
        let alpha = sp.num("opacity").unwrap_or(1.0) as f32;
        ops.push(Op::Text(TextRun {
            font,
            size,
            color: Color {
                a: color.a * alpha,
                ..color
            },
            x,
            y,
            glyphs,
            text: s,
        }));
    }
}

/// The first family of `fontFamily` that resolves, else Helvetica.
fn font(p: &Props, fonts: &FontStore) -> FontId {
    let weight = match p.get("fontWeight") {
        Some(Value::String(s)) => weight_from_name(s).or(s.parse().ok()).unwrap_or(400),
        Some(v) => v.as_f64().map_or(400, |w| w as u16),
        None => 400,
    };
    let italic = p.str("fontStyle") == Some("italic");
    let families: Vec<String> = match p.get("fontFamily") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    };
    families
        .iter()
        .map(String::as_str)
        .chain(["Helvetica"])
        .find_map(|f| fonts.resolve(f, weight, italic).ok())
        .unwrap_or(FontId(0))
}

fn transform(s: &str, how: Option<&str>) -> String {
    match how {
        Some("uppercase") => s.to_uppercase(),
        Some("lowercase") => s.to_lowercase(),
        _ => s.to_string(),
    }
}
