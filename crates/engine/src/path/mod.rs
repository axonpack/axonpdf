//! Path building: rounded boxes and ellipses.

use crate::display::{Rect, Seg};

/// Control-point distance for a quarter circle drawn as one cubic, as a share of the radius.
pub const KAPPA: f32 = 0.552_284_8;

/// A box with circular corners (top left, top right, bottom right, bottom left), each radius
/// clamped to half the width and half the height.
pub fn rounded_rect(r: Rect, radii: [f32; 4]) -> Vec<Seg> {
    let clamp = |v: f32| v.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    rounded_rect_xy(r, radii.map(|v| (clamp(v), clamp(v))))
}

/// A box with elliptical corners, `(rx, ry)` each, not clamped. Starts at the end of the top
/// left corner and runs clockwise.
pub fn rounded_rect_xy(r: Rect, radii: [(f32, f32); 4]) -> Vec<Seg> {
    let [tl, tr, br, bl] = radii;
    let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    let k = 1.0 - KAPPA;
    vec![
        Seg::Move(x0 + tl.0, y0),
        Seg::Line(x1 - tr.0, y0),
        Seg::Cubic(x1 - tr.0 * k, y0, x1, y0 + tr.1 * k, x1, y0 + tr.1),
        Seg::Line(x1, y1 - br.1),
        Seg::Cubic(x1, y1 - br.1 * k, x1 - br.0 * k, y1, x1 - br.0, y1),
        Seg::Line(x0 + bl.0, y1),
        Seg::Cubic(x0 + bl.0 * k, y1, x0, y1 - bl.1 * k, x0, y1 - bl.1),
        Seg::Line(x0, y0 + tl.1),
        Seg::Cubic(x0, y0 + tl.1 * k, x0 + tl.0 * k, y0, x0 + tl.0, y0),
        Seg::Close,
    ]
}

/// An ellipse as four cubics, starting at its rightmost point.
pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Vec<Seg> {
    let (ox, oy) = (rx * KAPPA, ry * KAPPA);
    vec![
        Seg::Move(cx + rx, cy),
        Seg::Cubic(cx + rx, cy + oy, cx + ox, cy + ry, cx, cy + ry),
        Seg::Cubic(cx - ox, cy + ry, cx - rx, cy + oy, cx - rx, cy),
        Seg::Cubic(cx - rx, cy - oy, cx - ox, cy - ry, cx, cy - ry),
        Seg::Cubic(cx + ox, cy - ry, cx + rx, cy - oy, cx + rx, cy),
        Seg::Close,
    ]
}

/// Applies `[a b c d e f]` to a point.
pub fn apply(m: [f32; 6], x: f32, y: f32) -> (f32, f32) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

/// `outer` then `inner`: the matrix that applies `inner` first.
pub fn multiply(outer: [f32; 6], inner: [f32; 6]) -> [f32; 6] {
    let [a, b, c, d, e, f] = outer;
    let [a2, b2, c2, d2, e2, f2] = inner;
    [
        a * a2 + c * b2,
        b * a2 + d * b2,
        a * c2 + c * d2,
        b * c2 + d * d2,
        a * e2 + c * f2 + e,
        b * e2 + d * f2 + f,
    ]
}
