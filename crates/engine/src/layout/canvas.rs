//! Yoga's measure for a canvas: the extent of what its `paint` function draws.

use serde_json::Value;

use super::Assets;
use super::yoga::{Measure, MeasureMode};
use crate::style::Dim;
use crate::tree::Node;

/// The furthest x and y the painter calls reach. Only shapes count: rectangles, moves and lines,
/// circles and ellipses, polygons.
fn extent(calls: &[Value]) -> (f64, f64) {
    let (mut w, mut h) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut add = |x: f64, y: f64| {
        w = w.max(x);
        h = h.max(y);
    };
    for call in calls {
        let Some(c) = call.as_array() else { continue };
        let n = |i: usize| c.get(i).and_then(Value::as_f64).unwrap_or(f64::NAN);
        match c.first().and_then(Value::as_str) {
            Some("rect" | "roundedRect") => add(n(1) + n(3), n(2) + n(4)),
            Some("moveTo" | "lineTo") => add(n(1), n(2)),
            Some("circle" | "ellipse") => {
                let rx = n(3);
                let ry = if n(4) != 0.0 && !n(4).is_nan() {
                    n(4)
                } else {
                    rx
                };
                add(n(1) + rx, n(2) + ry);
            }
            Some("polygon") => {
                for p in c.iter().skip(1).filter_map(Value::as_array) {
                    let at = |i: usize| p.get(i).and_then(Value::as_f64).unwrap_or(f64::NAN);
                    add(at(0), at(1));
                }
            }
            _ => {}
        }
    }
    (w, h)
}

/// Asks the paint function once, with no size, and measures what it drew; the height is kept
/// under the page's content height less the canvas's margins and 10pt.
pub fn measure(n: &Node, page_area: Option<f64>, a: &Assets) -> Option<Box<Measure>> {
    let calls: Vec<Value> = serde_json::from_str(&(a.paint)(n.paint?, f64::NAN, f64::NAN)?).ok()?;
    let (w, h) = extent(&calls);
    let margin = |i: usize| match n.style.margin[i] {
        Some(Dim::Pt(x)) => x,
        _ => 0.0,
    };
    let area = page_area.map_or(f64::INFINITY, |p| p - margin(0) - margin(2) - 10.0);
    let h = area.min(h);
    Some(Box::new(
        move |_: f32, _: MeasureMode, _: f32, _: MeasureMode| (w as f32, h as f32),
    ))
}
