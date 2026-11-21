//! Yoga's measure for an image: its aspect ratio, kept under the page height.

use super::Assets;
use super::yoga::{Measure, MeasureMode};
use crate::style::Dim;
use crate::tree::Node;

/// The asset id in an image node's `src` (`{"asset": id}`).
pub fn asset(n: &Node) -> Option<u32> {
    n.props.get("src")?.get("asset")?.as_u64().map(|v| v as u32)
}

/// Lets an image fit the page's content height (minus its margins and 10pt) when the page has
/// one. An image without data measures 0 by 0.
pub fn measure(n: &Node, page_area: Option<f64>, a: &Assets) -> Option<Box<Measure>> {
    let size = asset(n).and_then(|id| (a.image_size)(id));
    let margin = |i: usize| match n.style.margin[i] {
        Some(Dim::Pt(x)) => x,
        _ => 0.0,
    };
    let area = page_area.map_or(f64::INFINITY, |h| h - margin(0) - margin(2) - 10.0);
    Some(Box::new(move |w: f32, wm: MeasureMode, h: f32, hm: MeasureMode| {
        use MeasureMode::*;
        let Some((iw, ih)) = size else { return (0.0, 0.0) };
        let ratio = iw / ih;
        let (w, h) = (w as f64, h as f64);
        let (rw, rh) = match (wm, hm) {
            (Exactly, Undefined) => (f64::NAN, area.min(w / ratio)),
            (AtMost | Undefined, Exactly) => ((h * ratio).min(w), f64::NAN),
            (Exactly, AtMost) => (f64::NAN, h.min(area).min(w / ratio)),
            (AtMost, AtMost) if ratio > 1.0 => (w, (w / ratio).min(h)),
            (AtMost, AtMost) => ((h * ratio).min(w), h),
            _ => (w, h),
        };
        (rw as f32, rh as f32)
    }))
}
