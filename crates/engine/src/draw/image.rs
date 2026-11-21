//! An image in its box: `objectFit` and `objectPosition`, clipped to the box and its corners.

use super::rect;
use crate::display::{ImageId, Op};
use crate::path::rounded_rect;
use crate::style::{Dim, Style};

/// Where the image goes inside a `cw` by `ch` box: width, height and offsets.
pub fn fit(s: &Style, cw: f64, ch: f64, iw: f64, ih: f64) -> (f64, f64, f64, f64) {
    let [px, py] = s.object_position;
    // A percentage places the image by that share of the free space; points place it exactly.
    let pct = |d: Option<Dim>| match d {
        Some(Dim::Pct(p)) => p / 100.0,
        _ => 0.5,
    };
    let pt = |d: Option<Dim>| match d {
        Some(Dim::Pt(v)) => Some(v),
        _ => None,
    };
    let place = |w: f64, h: f64, x_free: bool, y_free: bool| {
        let x = pt(px).unwrap_or(if x_free { (cw - w) * pct(px) } else { 0.0 });
        let y = pt(py).unwrap_or(if y_free { (ch - h) * pct(py) } else { 0.0 });
        (w, h, x, y)
    };
    let (cr, ir) = (cw / ch, iw / ih);
    let contain = || if cr > ir { place(ch * ir, ch, true, false) } else { place(cw, cw / ir, false, true) };
    let none = || place(iw, ih, true, true);
    match s.object_fit.as_deref() {
        Some("contain") => contain(),
        Some("cover") => if cr > ir { place(cw, cw / ir, false, true) } else { place(ch * ir, ch, true, false) },
        Some("none") => none(),
        Some("scale-down") => {
            let (c, n) = (contain(), none());
            if c.0 < n.0 { c } else { n }
        }
        _ => (cw, ch, pt(px).unwrap_or(0.0), pt(py).unwrap_or(0.0)),
    }
}

/// Draws image `id` (`size` in pixels) into the content box of a node at (`x`, `y`).
#[allow(clippy::too_many_arguments)]
pub fn draw(ops: &mut Vec<Op>, s: &Style, padding: [f64; 4], radii: [f32; 4], id: ImageId, size: (f64, f64), x: f64, y: f64, w: f64, h: f64) {
    let [pt, pr, pb, pl] = padding;
    let (cw, ch) = (w - pl - pr, h - pt - pb);
    let (iw, ih, xo, yo) = fit(s, cw, ch, size.0, size.1);
    if iw == 0.0 || ih == 0.0 {
        return;
    }
    ops.push(Op::Save);
    ops.push(Op::Clip { path: rounded_rect(rect(x, y, w, h), radii), rule: Default::default() });
    let opacity = s.opacity.unwrap_or(1.0) as f32;
    ops.push(Op::Image { id, rect: rect(x + pl + xo, y + pt + yo, iw, ih), opacity });
    ops.push(Op::Restore);
}
