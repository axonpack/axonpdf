//! A page's header and footer bands (axonpdf's own): drawn across the full page width at its
//! top and bottom on every page, and moving the content in by their heights.

use std::rc::Rc;

use serde_json::{Value, json};

use crate::style::Dim;
use crate::tree::{Node, RawNode};

/// Marks a band node: `header` or `footer`.
pub const BAND: &str = "__band";

/// The page's `header` and `footer` props as nodes: `{"render": id}` for a function, else the
/// node as sent. They come out `fixed` and absolutely placed.
pub fn take(raw: &mut RawNode) -> Result<Vec<RawNode>, String> {
    let mut out = Vec::new();
    for which in ["header", "footer"] {
        let Some(v) = raw.props.shift_remove(which) else {
            continue;
        };
        let mut band: RawNode = match v.get("render").and_then(Value::as_u64) {
            Some(id) if v.get("type").is_none() => {
                let mut n: RawNode = serde_json::from_value(json!({"type": "VIEW"})).unwrap();
                n.render = Some(id as u32);
                n
            }
            _ => serde_json::from_value(v).map_err(|e| format!("page {which}: {e}"))?,
        };
        band.props.insert("fixed".into(), true.into());
        band.props.insert(BAND.into(), which.into());
        let edge = if which == "header" { "top" } else { "bottom" };
        let place = json!({"position": "absolute", "left": 0, "right": 0, edge: 0});
        band.style = json!([std::mem::take(&mut band.style), place]);
        out.push(band);
    }
    Ok(out)
}

/// Sets the page's top and bottom padding to its own plus its bands' heights. Returns whether
/// they changed. `base` is the page's own top and bottom padding.
pub fn fit(page: &mut Node, base: (f64, f64)) -> bool {
    let height_of = |which: &str| {
        page.children
            .iter()
            .filter(|c| c.props.get(BAND).and_then(Value::as_str) == Some(which))
            .map(|c| c.layout.height)
            .sum::<f64>()
    };
    let (top, bottom) = (base.0 + height_of("header"), base.1 + height_of("footer"));
    let now = (page.style.padding[0], page.style.padding[2]);
    if now == (Some(Dim::Pt(top)), Some(Dim::Pt(bottom))) {
        return false;
    }
    let s = Rc::make_mut(&mut page.style);
    s.padding[0] = Some(Dim::Pt(top));
    s.padding[2] = Some(Dim::Pt(bottom));
    page.dirty = true;
    true
}
