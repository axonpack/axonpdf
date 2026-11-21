//! `debug`: content, padding and margin tinted, the outer size printed, the origin marked.

use super::rect;
use crate::display::{Color, Op, Paint, Stroke, TextRun};
use crate::fonts::FontStore;
use crate::path::ellipse;
use crate::tree::Node;

fn tint(rgb: u32) -> Paint {
    let c = |s: u32| ((rgb >> s) & 0xff) as f32 / 255.0;
    Paint::Solid(Color { r: c(16), g: c(8), b: c(0), a: 0.5 })
}

fn fill(ops: &mut Vec<Op>, paint: &Paint, x: f64, y: f64, w: f64, h: f64) {
    ops.push(Op::Rect { rect: rect(x, y, w, h), radii: [0.0; 4], fill: Some(paint.clone()), stroke: None });
}

pub fn draw(ops: &mut Vec<Op>, n: &Node, x: f64, y: f64, origin: (f64, f64), fonts: &FontStore) {
    let l = &n.layout;
    let (w, h) = (l.width, l.height);
    let [pt, pr, pb, pl] = l.padding;
    let [bt, br, bb, bl] = l.border;
    let [mt, mr, mb, ml] = l.margin;
    fill(ops, &tint(0xa1c6e7), x + pl + bl, y + pt + bt, w - pl - pr - br - bl, h - pt - pb - bt - bb);
    let pad = tint(0xc4deb9);
    fill(ops, &pad, x + pl + bl, y + bt, w - pr - pl - bl - br, pt);
    fill(ops, &pad, x + bl, y + bt, pl, h - bt - bb);
    fill(ops, &pad, x + w - pr - br, y + bt, pr, h - bt - bb);
    fill(ops, &pad, x + pl + bl, y + h - pb - bb, w - pr - pl - bl - br, pb);
    let margin = tint(0xf8cca1);
    fill(ops, &margin, x, y - mt, w, mt);
    fill(ops, &margin, x - ml, y - mt, ml, h + mt + mb);
    fill(ops, &margin, x + w, y - mt, mr, h + mt + mb);
    fill(ops, &margin, x, y + h, w, mb);

    // The label sits on a 6pt Helvetica line whose top is 4pt above the margin box.
    let label = format!("{} x {}", (w + ml + mr).round(), (h + mt + mb).round());
    let font = fonts.resolve("Helvetica", 400, false).unwrap();
    let top = (y - mt - 4.0).max(1.0);
    ops.push(Op::Text(TextRun {
        font,
        size: 6.0,
        color: Color::BLACK,
        x: (x - ml) as f32,
        y: (top + 0.718 * 6.0) as f32,
        glyphs: fonts.shape(font, &label, 6.0, 0.0, &[]),
        text: label,
    }));

    let red = Paint::Solid(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
    let (ox, oy) = (origin.0 as f32, origin.1 as f32);
    ops.push(Op::Path { path: ellipse(ox, oy, 3.0, 3.0), fill: Some(crate::display::Fill { paint: red.clone(), rule: Default::default() }), stroke: None });
    ops.push(Op::Path {
        path: ellipse(ox, oy, 5.0, 5.0),
        fill: None,
        stroke: Some(Stroke { paint: red, width: 1.0, cap: Default::default(), join: Default::default(), miter_limit: 10.0, dash: vec![], dash_phase: 0.0 }),
    });
}
