//! A `Canvas` node's painter calls into display ops.
//!
//! The components run the user's `paint(painter, width, height)` against a painter that
//! records each call, and the engine receives them as a JSON array of `[method, ...args]`, in
//! call order, with the painter's own method names and arguments:
//!
//! - Path: `["moveTo", x, y]`, `["lineTo", x, y]`, `["bezierCurveTo", c1x, c1y, c2x, c2y, x, y]`,
//!   `["quadraticCurveTo", cx, cy, x, y]`, `["rect", x, y, w, h]`,
//!   `["roundedRect", x, y, w, h, r]`, `["ellipse", cx, cy, rx, ry?]`, `["circle", cx, cy, r]`,
//!   `["polygon", [x, y], [x, y], ...]`, `["path", "<svg path data>"]`.
//! - Painting the path (then starting a new one): `["fill", color?, rule?]`,
//!   `["stroke", color?]`, `["fillAndStroke", fill?, stroke?, rule?]`, `["clip", rule?]`. A rule
//!   is `"evenodd"`/`"even-odd"`, and `fill` also takes the rule in place of the color.
//! - State: `["save"]`, `["restore"]`, `["fillColor", color, opacity?]`,
//!   `["strokeColor", color, opacity?]`, `["opacity", a]`, `["fillOpacity", a]`,
//!   `["strokeOpacity", a]`, `["lineWidth", w]`, `["lineCap", "butt"|"round"|"square"]`,
//!   `["lineJoin", "miter"|"round"|"bevel"]`, `["miterLimit", m]`,
//!   `["dash", length | [lengths], {"space"?, "phase"?}?]`.
//! - Transforms: `["translate", x, y]`, `["rotate", degrees, {"origin": [x, y]}?]`,
//!   `["scale", sx, sy?, {"origin": [x, y]}?]` (`sy` may be the options).
//! - Text: `["font", family, size?]`, `["fontSize", size]`, `["text", string, x, y]`; the text
//!   hangs from `y` (its top), on one line.
//! - Gradients: `["linearGradient", id, x1, y1, x2, y2]`,
//!   `["radialGradient", id, x1, y1, r1, x2, y2, r2]`, `["stop", id, offset, color, opacity?]`;
//!   a color argument `{"gradient": id}` paints with one.
//!
//! Colors are CSS strings or `[r, g, b]` arrays of 0 to 255. Unknown calls are skipped.

mod state;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use serde_json::Value;

use crate::display::{
    Color, FillRule, FontId, Gradient, GradientKind, LineCap, LineJoin, Op, Paint, Seg, TextRun,
};
use crate::fonts::{Font, FontStore, standard};
use crate::path::{PathBuilder, ellipse, rounded_rect, svg};
use state::{State, add, color, origin, paint, set};

/// Draws `calls` with the origin at (`x`, `y`), the canvas's content box.
pub fn draw(ops: &mut Vec<Op>, calls: &[Value], x: f64, y: f64, fonts: &FontStore) {
    let black = Paint::Solid(Color::BLACK);
    let mut st = State {
        fill: black.clone(),
        stroke: black,
        fill_alpha: 1.0,
        stroke_alpha: 1.0,
        width: 1.0,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        miter: 10.0,
        dash: (Vec::new(), 0.0),
        font: FontId(0),
        size: 12.0,
    };
    let mut stack = Vec::new();
    let mut path = PathBuilder::new();
    let mut gradients: HashMap<u64, Gradient> = HashMap::new();
    ops.push(Op::Save);
    ops.push(Op::Transform([1.0, 0.0, 0.0, 1.0, x as f32, y as f32]));
    for call in calls {
        let Some((name, args)) = call
            .as_array()
            .and_then(|a| Some((a.first()?.as_str()?, &a[1..])))
        else {
            continue;
        };
        let n = |i: usize| args.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32;
        match name {
            "save" => {
                ops.push(Op::Save);
                stack.push(st.clone());
            }
            "restore" => {
                if let Some(s) = stack.pop() {
                    ops.push(Op::Restore);
                    st = s;
                }
            }
            "moveTo" => path.move_to(n(0), n(1)),
            "lineTo" => path.line_to(n(0), n(1)),
            "bezierCurveTo" => path.cubic_to(n(0), n(1), n(2), n(3), n(4), n(5)),
            // Drawn as a cubic whose first control point is the current point.
            "quadraticCurveTo" => {
                let (px, py) = path.pos();
                path.cubic_to(px, py, n(0), n(1), n(2), n(3));
            }
            "rect" => add(
                &mut path,
                &[
                    Seg::Move(n(0), n(1)),
                    Seg::Line(n(0) + n(2), n(1)),
                    Seg::Line(n(0) + n(2), n(1) + n(3)),
                    Seg::Line(n(0), n(1) + n(3)),
                    Seg::Close,
                ],
            ),
            "roundedRect" => {
                let r = crate::display::Rect {
                    x: n(0),
                    y: n(1),
                    w: n(2),
                    h: n(3),
                };
                add(&mut path, &rounded_rect(r, [n(4); 4]));
            }
            "ellipse" => add(
                &mut path,
                &ellipse(
                    n(0),
                    n(1),
                    n(2),
                    args.get(3)
                        .and_then(Value::as_f64)
                        .map_or(n(2), |v| v as f32),
                ),
            ),
            "circle" => add(&mut path, &ellipse(n(0), n(1), n(2), n(2))),
            "polygon" => {
                let pts: Vec<(f32, f32)> = args
                    .iter()
                    .filter_map(|p| Some((p.get(0)?.as_f64()? as f32, p.get(1)?.as_f64()? as f32)))
                    .collect();
                add(&mut path, &svg::polyline(&pts, true));
            }
            "path" => add(
                &mut path,
                &svg::parse(args.first().and_then(Value::as_str).unwrap_or("")),
            ),
            "fill" | "stroke" | "fillAndStroke" | "clip" => {
                let rule_of = |v: Option<&Value>| match v.and_then(Value::as_str) {
                    Some(s) if s.contains("odd") => FillRule::EvenOdd,
                    _ => FillRule::NonZero,
                };
                let is_rule = |v: Option<&Value>| {
                    v.and_then(Value::as_str)
                        .is_some_and(|s| s.contains("odd") || s.contains("zero"))
                };
                let segs = std::mem::take(&mut path.segs);
                path = PathBuilder::new();
                let (mut rule, mut fill, mut stroke) = (FillRule::NonZero, false, false);
                match name {
                    "fill" if is_rule(args.first()) => (rule, fill) = (rule_of(args.first()), true),
                    "fill" => {
                        set(&mut st.fill, args.first(), &gradients);
                        (rule, fill) = (rule_of(args.get(1)), true);
                    }
                    "stroke" => {
                        set(&mut st.stroke, args.first(), &gradients);
                        stroke = true;
                    }
                    "fillAndStroke" => {
                        set(&mut st.fill, args.first(), &gradients);
                        set(&mut st.stroke, args.get(1), &gradients);
                        (rule, fill, stroke) = (rule_of(args.get(2)), true, true);
                    }
                    _ => {
                        ops.push(Op::Clip {
                            path: segs,
                            rule: rule_of(args.first()),
                        });
                        continue;
                    }
                }
                paint(ops, &st, segs, fill.then_some(rule), stroke);
            }
            "fillColor" | "strokeColor" => {
                let fill = name == "fillColor";
                set(
                    if fill { &mut st.fill } else { &mut st.stroke },
                    args.first(),
                    &gradients,
                );
                if let Some(a) = args.get(1).and_then(Value::as_f64) {
                    *if fill {
                        &mut st.fill_alpha
                    } else {
                        &mut st.stroke_alpha
                    } = a as f32;
                }
            }
            "opacity" => (st.fill_alpha, st.stroke_alpha) = (n(0), n(0)),
            "fillOpacity" => st.fill_alpha = n(0),
            "strokeOpacity" => st.stroke_alpha = n(0),
            "lineWidth" => st.width = n(0),
            "lineCap" => {
                st.cap = match args.first().and_then(Value::as_str) {
                    Some("round") => LineCap::Round,
                    Some("square") => LineCap::Square,
                    _ => LineCap::Butt,
                }
            }
            "lineJoin" => {
                st.join = match args.first().and_then(Value::as_str) {
                    Some("round") => LineJoin::Round,
                    Some("bevel") => LineJoin::Bevel,
                    _ => LineJoin::Miter,
                }
            }
            "miterLimit" => st.miter = n(0),
            "dash" => {
                let opts = args.get(1);
                let num = |k: &str| {
                    opts.and_then(|o| o.get(k))
                        .and_then(Value::as_f64)
                        .map(|v| v as f32)
                };
                let lengths = match args.first() {
                    Some(Value::Array(a)) => a
                        .iter()
                        .filter_map(Value::as_f64)
                        .map(|v| v as f32)
                        .collect(),
                    _ => vec![n(0), num("space").unwrap_or(n(0))],
                };
                st.dash = (lengths, num("phase").unwrap_or(0.0));
            }
            "translate" => ops.push(Op::Transform([1.0, 0.0, 0.0, 1.0, n(0), n(1)])),
            "rotate" => {
                let (s, c) = n(0).to_radians().sin_cos();
                let (ox, oy) = origin(args.get(1));
                ops.push(Op::Transform([
                    c,
                    s,
                    -s,
                    c,
                    ox - (ox * c - oy * s),
                    oy - (ox * s + oy * c),
                ]));
            }
            "scale" => {
                let (sx, rest) = (n(0), args.get(1));
                let sy = rest.and_then(Value::as_f64).map_or(sx, |v| v as f32);
                let (ox, oy) = origin(if rest.is_some_and(Value::is_object) {
                    rest
                } else {
                    args.get(2)
                });
                ops.push(Op::Transform([
                    sx,
                    0.0,
                    0.0,
                    sy,
                    ox - sx * ox,
                    oy - sy * oy,
                ]));
            }
            "font" => {
                if let Some(family) = args.first().and_then(Value::as_str) {
                    let by_name = standard::NAMES
                        .iter()
                        .position(|s| *s == family)
                        .map(|i| FontId(i as u32));
                    st.font = by_name
                        .or_else(|| fonts.resolve(family, 400, false).ok())
                        .unwrap_or(st.font);
                }
                if let Some(size) = args.iter().skip(1).find_map(Value::as_f64) {
                    st.size = size as f32;
                }
            }
            "fontSize" => st.size = n(0),
            "text" => {
                let s = args
                    .first()
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let ascender = match fonts.font(st.font) {
                    Font::Standard(i) => standard::get(*i).afm_ascender,
                    Font::Embedded(e) => e.metrics.ascent * 1000.0 / e.metrics.units_per_em,
                };
                let color = match &st.fill {
                    Paint::Solid(c) => Color {
                        a: c.a * st.fill_alpha,
                        ..*c
                    },
                    Paint::Gradient(_) => Color::BLACK,
                };
                let glyphs = fonts.shape(st.font, &s, st.size, 0.0, &[]);
                let y = n(2) + ascender / 1000.0 * st.size;
                ops.push(Op::Text(TextRun {
                    font: st.font,
                    size: st.size,
                    color,
                    x: n(1),
                    y,
                    glyphs,
                    text: s,
                }));
            }
            "linearGradient" | "radialGradient" => {
                let id = args.first().and_then(Value::as_u64).unwrap_or(0);
                let kind = if name == "linearGradient" {
                    GradientKind::Linear {
                        x1: n(1),
                        y1: n(2),
                        x2: n(3),
                        y2: n(4),
                    }
                } else {
                    GradientKind::Radial {
                        fx: n(1),
                        fy: n(2),
                        fr: n(3),
                        cx: n(4),
                        cy: n(5),
                        r: n(6),
                    }
                };
                gradients.insert(
                    id,
                    Gradient {
                        kind,
                        stops: Vec::new(),
                        transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    },
                );
            }
            "stop" => {
                let id = args.first().and_then(Value::as_u64).unwrap_or(0);
                if let Some(g) = gradients.get_mut(&id) {
                    let c = color(args.get(2)).unwrap_or(Color::BLACK);
                    let a = args.get(3).and_then(Value::as_f64).unwrap_or(1.0) as f32;
                    g.stops.push((n(1), Color { a: c.a * a, ..c }));
                }
            }
            _ => {}
        }
    }
    ops.push(Op::Restore);
}
