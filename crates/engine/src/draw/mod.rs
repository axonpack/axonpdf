//! A finished page's nodes into the display list: backgrounds, borders, text, images, links
//! and destinations, in paint order.

mod debug;
mod image;
mod svg;
mod text;
mod transform;

use crate::display::{Border, BorderStyle, Color, ImageId, Op, PageOut, Paint, Rect};
use crate::fonts::FontStore;
use crate::layout::image::asset;
use crate::path::rounded_rect;
use crate::style::{Dim, Style};
use crate::tree::{Kind, Node};

/// The stored image and its size in pixels, for a document asset id.
pub type ImageLookup<'a> = dyn Fn(u32) -> Option<(ImageId, (f64, f64))> + 'a;

/// What drawing needs besides the page.
pub struct Ctx<'a> {
    pub fonts: &'a FontStore,
    pub images: &'a ImageLookup<'a>,
}

pub fn page(page: &Node, width: f64, height: f64, cx: &Ctx) -> PageOut {
    let mut ops = Vec::new();
    node(&mut ops, page, 0.0, 0.0, cx);
    PageOut {
        width: width as f32,
        height: height as f32,
        ops,
    }
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect {
        x: x as f32,
        y: y as f32,
        w: w as f32,
        h: h as f32,
    }
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
    Color {
        a: c.a.min(a as f32),
        ..c
    }
}

/// Draws `n`, whose parent's border box starts at (`ox`, `oy`).
fn node(ops: &mut Vec<Op>, n: &Node, ox: f64, oy: f64, cx: &Ctx) {
    let s = &n.style;
    let l = &n.layout;
    let (x, y, w, h) = (ox + l.left, oy + l.top, l.width, l.height);
    let opacity = s.opacity.unwrap_or(1.0);
    let r = radii(s, w, h);
    let origin = transform::origin(s, x, y, w, h);
    ops.push(Op::Save);
    if s.overflow == Some(crate::layout::yoga::Overflow::Hidden) {
        ops.push(Op::Clip {
            path: rounded_rect(rect(x, y, w, h), r),
            rule: Default::default(),
        });
    }
    ops.extend(transform::ops(s, origin));
    if let Some(bg) = s.background_color {
        ops.push(Op::Rect {
            rect: rect(x, y, w, h),
            radii: r,
            fill: Some(Paint::Solid(with_alpha(bg, opacity))),
            stroke: None,
        });
    }
    if l.border.iter().any(|b| *b > 0.0) {
        let colors = s
            .border_color
            .map(|c| with_alpha(c.unwrap_or(Color::BLACK), opacity));
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
        Kind::Svg => svg::draw(ops, n, x, y, r, cx),
        Kind::Image | Kind::ImageBackground => {
            if let Some((id, size)) = asset(n).and_then(cx.images) {
                image::draw(ops, s, l.padding, r, id, size, x, y, w, h);
            }
        }
        Kind::Link => {
            let href = n
                .props
                .get("src")
                .or(n.props.get("href"))
                .and_then(|v| v.as_str());
            if let Some(href) = href {
                ops.push(Op::Link {
                    rect: rect(x, y, w, h),
                    target: text::target(href),
                });
            }
        }
        _ => {}
    }
    if !matches!(n.kind, Kind::Text | Kind::Svg) {
        for child in by_z_index(&n.children) {
            node(ops, child, x, y, cx);
        }
    }
    if let Some(id) = n.props.get("id").and_then(|v| v.as_str()) {
        ops.push(Op::Dest {
            name: id.to_string(),
            x: x as f32,
            y: y as f32,
        });
    }
    if n.is("debug") {
        debug::draw(ops, n, x, y, origin, cx.fonts);
    }
    ops.push(Op::Restore);
}

/// Paint order: children with a `zIndex` first, highest first, then the rest as written.
fn by_z_index(children: &[Node]) -> Vec<&Node> {
    let mut v: Vec<&Node> = children.iter().filter(|c| c.kind != Kind::TextInstance).collect();
    let z = |n: &Node| n.style.z_index.filter(|z| *z != 0.0 && !z.is_nan());
    v.sort_by(|a, b| match (z(a), z(b)) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, _) => std::cmp::Ordering::Greater,
        (_, None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => b.total_cmp(&a),
    });
    v
}
