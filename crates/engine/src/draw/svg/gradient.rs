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
            // From the focal point out to the circle at the centre, as SVG defines it.
            GradientKind::Radial {
                fx: f(fx),
                fy: f(fy),
                fr: 0.0,
                cx: f(cx),
                cy: f(cy),
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

/// `gradientTransform`, read like any SVG transform (angles in degrees, or `rad`).
/// Translations in bounding-box units scale with the box.
fn matrix(p: &Props, (w, h): (f64, f64), object: bool) -> [f32; 6] {
    let ops = p
        .get("gradientTransform")
        .and_then(transform::parse)
        .unwrap_or_default();
    let mut m = [1.0f32, 0.0, 0.0, 1.0, 0.0, 0.0];
    for t in ops {
        let mut step = super::matrix(&t);
        if object && matches!(t.op.as_str(), "translate" | "matrix") {
            step[4] *= w as f32;
            step[5] *= h as f32;
        }
        m = multiply(m, step);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(json: serde_json::Value) -> Node {
        let raw = serde_json::from_value(json).unwrap();
        Node::build(raw, &crate::style::Container::default(), None)
    }

    #[test]
    fn radial_runs_from_the_focal_point_to_the_centre_circle() {
        let g = def(serde_json::json!({"type": "RADIAL_GRADIENT",
            "props": {"cx": "0.5", "cy": "0.5", "r": "0.5", "fx": "0.3", "fy": "0.2"}}));
        let Some(Paint::Gradient(g)) = paint(&g, [0.0, 0.0, 100.0, 100.0]) else {
            panic!()
        };
        let GradientKind::Radial {
            fx,
            fy,
            fr,
            cx,
            cy,
            r,
        } = g.kind
        else {
            panic!()
        };
        assert_eq!((fx, fy, fr, cx, cy, r), (30.0, 20.0, 0.0, 50.0, 50.0, 50.0));
    }

    #[test]
    fn transform_angles_are_degrees() {
        let g = def(serde_json::json!({"type": "LINEAR_GRADIENT",
            "props": {"gradientTransform": "rotate(90) skewX(45)"}}));
        let Some(Paint::Gradient(g)) = paint(&g, [0.0, 0.0, 10.0, 10.0]) else {
            panic!()
        };
        let m = g.transform.map(|v| (v * 1000.0).round() / 1000.0);
        // rotate(90) then skewX(45): [0 1 -1 0] x [1 0 1 1].
        assert_eq!(m, [0.0, 1.0, -1.0, 1.0, 0.0, 0.0]);
        let rad = def(
            serde_json::json!({"type": "LINEAR_GRADIENT", "props": {"gradientTransform": "rotate(1.5707964rad)"}}),
        );
        let Some(Paint::Gradient(rad)) = paint(&rad, [0.0, 0.0, 10.0, 10.0]) else {
            panic!()
        };
        assert!((rad.transform[1] - 1.0).abs() < 1e-5);
    }
}
