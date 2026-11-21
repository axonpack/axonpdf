//! `LinearGradient` and `RadialGradient` definitions as paints for one shape.

use super::props::Props;
use crate::display::{Color, Gradient, GradientKind, Paint};
use crate::path::multiply;
use crate::style::transform;
use crate::tree::{Kind, Node};

/// The gradient `def` paints, sized to `bbox` (`[x0, y0, x1, y1]`) when its units are the
/// bounding box (the default). `None` when `def` is not a gradient.
pub fn paint(def: &Node, bbox: [f64; 4]) -> Option<Paint> {
    let Kind::Other(kind) = &def.kind else {
        return None;
    };
    // Gradient coordinates are fractions or plain numbers, never page percentages.
    let p = Props::of(def, &Default::default(), (1.0, 1.0));
    let object = p.str("gradientUnits") != Some("userSpaceOnUse");
    let (w, h) = (bbox[2] - bbox[0], bbox[3] - bbox[1]);
    let to_box = |x: f64, y: f64| {
        if object {
            (w * x + bbox[0], h * y + bbox[1])
        } else {
            (x, y)
        }
    };
    let nonzero = |key: &str, default: f64| p.num(key).filter(|v| *v != 0.0).unwrap_or(default);
    let f = |v: f64| v as f32;
    let kind = match kind.as_str() {
        "LINEAR_GRADIENT" => {
            let (x1, y1) = to_box(p.n("x1"), p.n("y1"));
            let (x2, y2) = to_box(nonzero("x2", 1.0), p.n("y2"));
            GradientKind::Linear {
                x1: f(x1),
                y1: f(y1),
                x2: f(x2),
                y2: f(y2),
            }
        }
        "RADIAL_GRADIENT" => {
            let (cx0, cy0) = (nonzero("cx", 0.5), nonzero("cy", 0.5));
            let (cx, cy) = to_box(cx0, cy0);
            let (fx, fy) = to_box(nonzero("fx", cx0), nonzero("fy", cy0));
            let r = nonzero("r", 0.5) * if object { w } else { 1.0 };
            // The inner circle is the centre and the outer one sits on the focal point, as
            // documents have always been drawn.
            GradientKind::Radial {
                fx: f(cx),
                fy: f(cy),
                fr: 0.0,
                cx: f(fx),
                cy: f(fy),
                r: f(r),
            }
        }
        _ => return None,
    };
    let stops = def
        .children
        .iter()
        .map(|stop| {
            let s = Props::of(stop, &Default::default(), (1.0, 1.0));
            let color = s.color("stopColor").unwrap_or(Color::BLACK);
            let alpha = s.num("stopOpacity").unwrap_or(1.0) as f32;
            (
                s.n("offset") as f32,
                Color {
                    a: color.a * alpha,
                    ..color
                },
            )
        })
        .collect();
    let transform = matrix(&p, (w, h), object);
    Some(Paint::Gradient(Box::new(Gradient {
        kind,
        stops,
        transform,
    })))
}

/// `gradientTransform`. Rotation angles are read as radians and bounding-box translations
/// scale with the box, matching how documents have always rendered.
fn matrix(p: &Props, (w, h): (f64, f64), object: bool) -> [f32; 6] {
    let ops = p
        .get("gradientTransform")
        .and_then(transform::parse)
        .unwrap_or_default();
    let mut m = [1.0f32, 0.0, 0.0, 1.0, 0.0, 0.0];
    for t in ops {
        let v = |i: usize| t.values.get(i).copied().unwrap_or(0.0);
        let scale_xy = |x: f64, y: f64| if object { (w * x, h * y) } else { (x, y) };
        let step: [f64; 6] = match t.op.as_str() {
            "scale" => [v(0), 0.0, 0.0, v(1), 0.0, 0.0],
            "translate" => {
                let (x, y) = scale_xy(v(0), v(1));
                [1.0, 0.0, 0.0, 1.0, x, y]
            }
            "rotate" => {
                // The parser gives degrees; reading them as radians keeps old output.
                let (s, c) = v(0).sin_cos();
                let (cx, cy) = (v(1), v(2));
                [
                    c,
                    s,
                    -s,
                    c,
                    cx * (1.0 - c) + cy * s,
                    cy * (1.0 - c) - cx * s,
                ]
            }
            "skew" => [1.0, v(0).tan(), v(1).tan(), 1.0, 0.0, 0.0],
            "matrix" => {
                let (x, y) = scale_xy(v(4), v(5));
                [v(0), v(1), v(2), v(3), x, y]
            }
            _ => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        };
        m = multiply(m, step.map(|x| x as f32));
    }
    m
}
