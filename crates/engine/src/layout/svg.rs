//! Yoga's measure for an Svg: its viewBox's aspect ratio (1 without one).

use serde_json::Value;

use super::yoga::{Measure, MeasureMode};
use crate::tree::Node;

/// Width over height of `"minX minY maxX maxY"`, read as the components read it: the third
/// and fourth numbers less the first two.
fn ratio(n: &Node) -> f64 {
    let Some(Value::String(v)) = n.props.get("viewBox") else {
        return 1.0;
    };
    let nums: Vec<f64> = v
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap_or(f64::NAN))
        .collect();
    match nums.as_slice() {
        [x0, y0, x1, y1] => {
            let r = (x1 - x0) / (y1 - y0);
            if r == 0.0 || r.is_nan() { 1.0 } else { r }
        }
        _ => 1.0,
    }
}

pub fn measure(n: &Node) -> Box<Measure> {
    let r = ratio(n);
    Box::new(move |w: f32, wm: MeasureMode, h: f32, hm: MeasureMode| {
        let (w, h) = (w as f64, h as f64);
        let (rw, rh) = match (wm, hm) {
            (MeasureMode::Exactly | MeasureMode::AtMost, _) => (w, w / r),
            (_, MeasureMode::Exactly) => (h * r, f64::NAN),
            _ => (f64::NAN, f64::NAN),
        };
        (rw as f32, rh as f32)
    })
}
