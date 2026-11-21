//! A finished page's nodes into the display list: backgrounds, borders, text, images, links
//! and destinations, in paint order.

mod text;

use crate::display::{Border, BorderStyle, Color, Op, PageOut, Paint, Rect, Seg};
use crate::layout::image::asset;
use crate::style::{Dim, Style};
use crate::tree::{Kind, Node};

/// The image to draw for a document asset id.
pub type ImageLookup<'a> = dyn Fn(u32) -> Option<crate::display::ImageId> + 'a;

pub fn page(page: &Node, width: f64, height: f64, images: &ImageLookup) -> PageOut {
    let mut ops = Vec::new();
    node(&mut ops, page, 0.0, 0.0, images);
    PageOut { width: width as f32, height: height as f32, ops }
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect { x: x as f32, y: y as f32, w: w as f32, h: h as f32 }
}

/// Corner radii in points; percentages are of the shorter side.
fn radii(s: &Style, w: f64, h: f64) -> [f32; 4] {
    s.border_radius.map(|r| match r {
        Some(Dim::Pt(x)) => x as f32,
        Some(Dim::Pct(p)) => (p / 100.0 * w.min(h)) as f32,
        _ => 0.0,
    })
}

fn with_alpha(c: Color, a: f64) -> Color {
    Color { a: c.a.min(a as f32), ..c }
}

/// Draws `n`, whose parent's border box starts at (`ox`, `oy`).
fn node(ops: &mut Vec<Op>, n: &Node, ox: f64, oy: f64, images: &ImageLookup) {
    let s = &n.style;
    let l = &n.layout;
    let (x, y, w, h) = (ox + l.left, oy + l.top, l.width, l.height);
    let opacity = s.opacity.unwrap_or(1.0);
    let r = radii(s, w, h);
    ops.push(Op::Save);
    if s.overflow == Some(crate::layout::yoga::Overflow::Hidden) {
        ops.push(Op::Clip { path: rounded(x, y, w, h, r), rule: Default::default() });
    }
    if let Some(bg) = s.background_color {
        ops.push(Op::Rect { rect: rect(x, y, w, h), radii: r, fill: Some(Paint::Solid(with_alpha(bg, opacity))), stroke: None });
    }
    if l.border.iter().any(|b| *b > 0.0) {
        let colors = s.border_color.map(|c| with_alpha(c.unwrap_or(Color::BLACK), opacity));
        ops.push(Op::Border(Border {
            rect: rect(x, y, w, h),
            widths: l.border.map(|b| b as f32),
            colors,
            styles: s.border_style.map(|b| b.unwrap_or(BorderStyle::Solid)),
            radii: r,
        }));
    }
    match n.kind {
        Kind::Text => text::draw(ops, n, x, y),
        Kind::Image => {
            if let Some(id) = asset(n).and_then(images) {
                let p = l.padding;
                let area = rect(x + p[3], y + p[0], w - p[1] - p[3], h - p[0] - p[2]);
                ops.push(Op::Image { id, rect: area, opacity: opacity as f32 });
            }
        }
        Kind::Link => {
            let href = n.props.get("src").or(n.props.get("href")).and_then(|v| v.as_str());
            if let Some(href) = href {
                ops.push(Op::Link { rect: rect(x, y, w, h), target: text::target(href) });
            }
        }
        _ => {}
    }
    if !matches!(n.kind, Kind::Text | Kind::Svg) {
        for child in &n.children {
            if child.kind != Kind::TextInstance {
                node(ops, child, x, y, images);
            }
        }
    }
    if let Some(id) = n.props.get("id").and_then(|v| v.as_str()) {
        ops.push(Op::Dest { name: id.to_string(), x: x as f32, y: y as f32 });
    }
    ops.push(Op::Restore);
}

/// A rectangle with rounded corners, the corners drawn as cubic quarter circles.
fn rounded(x: f64, y: f64, w: f64, h: f64, r: [f32; 4]) -> Vec<Seg> {
    const KAPPA: f64 = 4.0 * ((std::f64::consts::SQRT_2 - 1.0) / 3.0);
    let r = r.map(|v| (v as f64).min(0.5 * w).min(0.5 * h));
    let c = r.map(|v| v * (1.0 - KAPPA));
    let f = |v: f64| v as f32;
    vec![
        Seg::Move(f(x + r[0]), f(y)),
        Seg::Line(f(x + w - r[1]), f(y)),
        Seg::Cubic(f(x + w - c[1]), f(y), f(x + w), f(y + c[1]), f(x + w), f(y + r[1])),
        Seg::Line(f(x + w), f(y + h - r[2])),
        Seg::Cubic(f(x + w), f(y + h - c[2]), f(x + w - c[2]), f(y + h), f(x + w - r[2]), f(y + h)),
        Seg::Line(f(x + r[3]), f(y + h)),
        Seg::Cubic(f(x + c[3]), f(y + h), f(x), f(y + h - c[3]), f(x), f(y + h - r[3])),
        Seg::Line(f(x), f(y + r[0])),
        Seg::Cubic(f(x), f(y + c[0]), f(x + c[0]), f(y), f(x + r[0]), f(y)),
        Seg::Close,
    ]
}
