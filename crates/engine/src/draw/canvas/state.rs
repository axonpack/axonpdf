//! The painter's state and painting one path.

use std::collections::HashMap;

use serde_json::Value;

use crate::display::{
    Color, Fill, FillRule, FontId, Gradient, LineCap, LineJoin, Op, Paint, Seg, Stroke,
};
use crate::path::PathBuilder;

#[derive(Clone)]
pub struct State {
    pub fill: Paint,
    pub stroke: Paint,
    pub fill_alpha: f32,
    pub stroke_alpha: f32,
    pub width: f32,
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter: f32,
    pub dash: (Vec<f32>, f32),
    pub font: FontId,
    pub size: f32,
}

pub fn add(path: &mut PathBuilder, segs: &[Seg]) {
    for s in segs {
        match *s {
            Seg::Move(x, y) => path.move_to(x, y),
            Seg::Line(x, y) => path.line_to(x, y),
            Seg::Cubic(a, b, c, d, x, y) => path.cubic_to(a, b, c, d, x, y),
            Seg::Close => path.close(),
        }
    }
}

pub fn origin(opts: Option<&Value>) -> (f32, f32) {
    let o = opts.and_then(|o| o.get("origin"));
    let at = |i: usize| {
        o.and_then(|o| o.get(i))
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32
    };
    (at(0), at(1))
}

pub fn color(v: Option<&Value>) -> Option<Color> {
    match v? {
        Value::String(s) => Some(crate::style::color::parse(s)),
        Value::Array(a) => {
            let c = |i: usize| a.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32 / 255.0;
            Some(Color {
                r: c(0),
                g: c(1),
                b: c(2),
                a: 1.0,
            })
        }
        _ => None,
    }
}

/// Sets a paint from a color argument; a missing or unreadable one leaves it as it was.
pub fn set(paint: &mut Paint, v: Option<&Value>, gradients: &HashMap<u64, Gradient>) {
    if let Some(id) = v.and_then(|v| v.get("gradient")).and_then(Value::as_u64) {
        if let Some(g) = gradients.get(&id) {
            *paint = Paint::Gradient(Box::new(g.clone()));
        }
    } else if let Some(c) = color(v) {
        *paint = Paint::Solid(c);
    }
}

pub fn paint(ops: &mut Vec<Op>, st: &State, path: Vec<Seg>, fill: Option<FillRule>, stroke: bool) {
    let with = |p: &Paint, a: f32| match p {
        Paint::Solid(c) => Paint::Solid(Color { a: c.a * a, ..*c }),
        g => g.clone(),
    };
    let fill = fill.map(|rule| Fill {
        paint: with(&st.fill, st.fill_alpha),
        rule,
    });
    let stroke = stroke.then(|| Stroke {
        paint: with(&st.stroke, st.stroke_alpha),
        width: st.width,
        cap: st.cap,
        join: st.join,
        miter_limit: st.miter,
        dash: st.dash.0.clone(),
        dash_phase: st.dash.1,
    });
    // Gradients take the alpha from the graphics state.
    let gradient_alpha = [
        (fill.as_ref().map(|f| &f.paint), st.fill_alpha),
        (stroke.as_ref().map(|s| &s.paint), st.stroke_alpha),
    ]
    .into_iter()
    .find_map(|(p, a)| matches!(p, Some(Paint::Gradient(_))).then_some(a))
    .filter(|a| *a < 1.0);
    if let Some(a) = gradient_alpha {
        ops.extend([Op::Save, Op::Opacity(a)]);
    }
    ops.push(Op::Path { path, fill, stroke });
    if gradient_alpha.is_some() {
        ops.push(Op::Restore);
    }
}
