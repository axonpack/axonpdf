//! A node's `transform`, applied around its transform origin.

use crate::display::Op;
use crate::style::{Dim, Style};

/// The transform origin in page coordinates: 50% of each side unless set.
pub fn origin(s: &Style, x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let at = |d: Option<Dim>, size: f64| match d {
        Some(Dim::Pt(v)) => v,
        Some(Dim::Pct(p)) => size * p / 100.0,
        _ => size / 2.0,
    };
    (x + at(s.transform_origin[0], w), y + at(s.transform_origin[1], h))
}

/// The ops for each transform operation, in order.
pub fn ops(s: &Style, (ox, oy): (f64, f64)) -> Vec<Op> {
    let mut out = Vec::new();
    let mut push = |m: [f64; 6]| out.push(Op::Transform(m.map(|v| v as f32)));
    for t in s.transform.iter().flatten() {
        let v = |i: usize| t.values.get(i).copied().unwrap_or(0.0);
        match t.op.as_str() {
            "scale" => push([v(0), 0.0, 0.0, v(1), ox - v(0) * ox, oy - v(1) * oy]),
            "rotate" => {
                let (cx, cy) = (v(1), v(2));
                let r = v(0).to_radians();
                let (cos, sin) = (r.cos(), r.sin());
                push([1.0, 0.0, 0.0, 1.0, cx, cy]);
                let (x1, y1) = (ox * cos - oy * sin, ox * sin + oy * cos);
                push([cos, sin, -sin, cos, ox - x1, oy - y1]);
                push([1.0, 0.0, 0.0, 1.0, -cx, -cy]);
            }
            "translate" => push([1.0, 0.0, 0.0, 1.0, v(0), v(1)]),
            "skew" => {
                let (tx, ty) = (v(0).to_radians().tan(), v(1).to_radians().tan());
                push([1.0, ty, tx, 1.0, -(tx * oy), -(ty * ox)]);
            }
            "matrix" => push([v(0), v(1), v(2), v(3), v(4), v(5)]),
            other => eprintln!("Transform operation: '{other}' doesn't supported"),
        }
    }
    out
}
