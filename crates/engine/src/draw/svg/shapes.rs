//! The outline of each SVG shape, and its bounding box (for gradients in bounding-box units).

use super::props::Props;
use crate::display::Seg;
use crate::path::{ellipse, svg};
use crate::tree::{Kind, Node};

/// The shape's path; `None` for nodes that are not shapes.
pub fn outline(node: &Node, p: &Props) -> Option<Vec<Seg>> {
    let Kind::Other(kind) = &node.kind else {
        return None;
    };
    let f = |v: f64| v as f32;
    Some(match kind.as_str() {
        "RECT" => {
            let (rx, ry) = (p.num("rx"), p.num("ry"));
            // One radius set means both.
            let (rx, ry) = (rx.or(ry).unwrap_or(0.0), ry.or(rx).unwrap_or(0.0));
            svg::rect(
                f(p.n("x")),
                f(p.n("y")),
                f(p.n("width")),
                f(p.n("height")),
                f(rx),
                f(ry),
            )
        }
        "CIRCLE" => ellipse(f(p.n("cx")), f(p.n("cy")), f(p.n("r")), f(p.n("r"))),
        "ELLIPSE" => ellipse(f(p.n("cx")), f(p.n("cy")), f(p.n("rx")), f(p.n("ry"))),
        "LINE" => vec![
            Seg::Move(f(p.n("x1")), f(p.n("y1"))),
            Seg::Line(f(p.n("x2")), f(p.n("y2"))),
        ],
        "POLYLINE" => svg::polyline(&points(p), false),
        "POLYGON" => svg::polyline(&points(p), true),
        "PATH" => svg::parse(p.str("d").unwrap_or("")),
        _ => return None,
    })
}

pub fn points(p: &Props) -> Vec<(f32, f32)> {
    svg::points(p.str("points").unwrap_or(""))
}

/// `[x0, y0, x1, y1]` of the shape's own coordinates (control points included for paths).
pub fn bbox(node: &Node, p: &Props) -> [f64; 4] {
    let Kind::Other(kind) = &node.kind else {
        return [0.0; 4];
    };
    match kind.as_str() {
        "RECT" => [
            p.n("x"),
            p.n("y"),
            p.n("x") + p.n("width"),
            p.n("y") + p.n("height"),
        ],
        "CIRCLE" => {
            let (cx, cy, r) = (p.n("cx"), p.n("cy"), p.n("r"));
            [cx - r, cy - r, cx + r, cy + r]
        }
        "ELLIPSE" => {
            let (cx, cy, rx, ry) = (p.n("cx"), p.n("cy"), p.n("rx"), p.n("ry"));
            [cx - rx, cy - ry, cx + rx, cy + ry]
        }
        _ => {
            let segs = outline(node, p).unwrap_or_default();
            let mut b = [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ];
            let mut add = |x: f32, y: f32| {
                b = [
                    b[0].min(x as f64),
                    b[1].min(y as f64),
                    b[2].max(x as f64),
                    b[3].max(y as f64),
                ];
            };
            for s in segs {
                match s {
                    Seg::Move(x, y) | Seg::Line(x, y) => add(x, y),
                    Seg::Cubic(a, b2, c, d, x, y) => {
                        add(a, b2);
                        add(c, d);
                        add(x, y);
                    }
                    Seg::Close => {}
                }
            }
            if b[0].is_finite() { b } else { [0.0; 4] }
        }
    }
}
