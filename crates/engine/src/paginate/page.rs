//! A `<Page>`: its size, its styles, and the values that must be known before layout.

use std::rc::Rc;

use serde_json::{Value, json};

use crate::style::units::{js_round, split_number};
use crate::style::{Container, Dim, flatten};
use crate::tree::{Node, RawNode, truthy};

/// Paper sizes in points.
const SIZES: [(&str, f64, f64); 51] = [
    ("4A0", 4767.87, 6740.79),
    ("2A0", 3370.39, 4767.87),
    ("A0", 2383.94, 3370.39),
    ("A1", 1683.78, 2383.94),
    ("A2", 1190.55, 1683.78),
    ("A3", 841.89, 1190.55),
    ("A4", 595.28, 841.89),
    ("A5", 419.53, 595.28),
    ("A6", 297.64, 419.53),
    ("A7", 209.76, 297.64),
    ("A8", 147.4, 209.76),
    ("A9", 104.88, 147.4),
    ("A10", 73.7, 104.88),
    ("B0", 2834.65, 4008.19),
    ("B1", 2004.09, 2834.65),
    ("B2", 1417.32, 2004.09),
    ("B3", 1000.63, 1417.32),
    ("B4", 708.66, 1000.63),
    ("B5", 498.9, 708.66),
    ("B6", 354.33, 498.9),
    ("B7", 249.45, 354.33),
    ("B8", 175.75, 249.45),
    ("B9", 124.72, 175.75),
    ("B10", 87.87, 124.72),
    ("C0", 2599.37, 3676.54),
    ("C1", 1836.85, 2599.37),
    ("C2", 1298.27, 1836.85),
    ("C3", 918.43, 1298.27),
    ("C4", 649.13, 918.43),
    ("C5", 459.21, 649.13),
    ("C6", 323.15, 459.21),
    ("C7", 229.61, 323.15),
    ("C8", 161.57, 229.61),
    ("C9", 113.39, 161.57),
    ("C10", 79.37, 113.39),
    ("RA0", 2437.8, 3458.27),
    ("RA1", 1729.13, 2437.8),
    ("RA2", 1218.9, 1729.13),
    ("RA3", 864.57, 1218.9),
    ("RA4", 609.45, 864.57),
    ("SRA0", 2551.18, 3628.35),
    ("SRA1", 1814.17, 2551.18),
    ("SRA2", 1275.59, 1814.17),
    ("SRA3", 907.09, 1275.59),
    ("SRA4", 637.8, 907.09),
    ("EXECUTIVE", 521.86, 756.0),
    ("FOLIO", 612.0, 936.0),
    ("LEGAL", 612.0, 1008.0),
    ("LETTER", 612.0, 792.0),
    ("TABLOID", 792.0, 1224.0),
    ("ID1", 153.0, 243.0),
];

/// A page dimension: points, `in`, `mm`, `cm` or `px` (at the page's dpi). `None` is auto.
fn length(v: &Value, dpi: f64) -> Result<Option<f64>, String> {
    let s = match v {
        Value::Number(n) => return Ok(n.as_f64().filter(|x| *x != 0.0)),
        Value::String(s) if s.is_empty() || s == "auto" => return Ok(None),
        Value::String(s) => s,
        _ => return Ok(None),
    };
    let (n, unit) = split_number(s).ok_or_else(|| format!("Invalid page size: {s}"))?;
    Ok(Some(match unit {
        "" | "pt" => n,
        "in" => n * 72.0,
        "mm" => n * 72.0 / 25.4,
        "cm" => n * 72.0 / 2.54,
        "px" => js_round(n * 72.0 / dpi),
        _ => return Err(format!("Invalid page size: {s}")),
    }))
}

/// Width and height (None: grows with content) of a page, orientation applied.
pub fn size(raw: &RawNode) -> Result<(f64, Option<f64>, bool), String> {
    let dpi = raw
        .props
        .get("dpi")
        .and_then(Value::as_f64)
        .filter(|d| *d != 0.0)
        .unwrap_or(72.0);
    let size = raw
        .props
        .get("size")
        .filter(|v| truthy(v))
        .cloned()
        .unwrap_or(json!("A4"));
    let (w, h) = match &size {
        Value::String(s) => {
            let up = s.to_uppercase();
            let (_, w, h) = SIZES
                .iter()
                .find(|(n, ..)| *n == up)
                .ok_or_else(|| format!("Invalid page size: {s}"))?;
            (*w, Some(*h))
        }
        Value::Array(a) => (
            length(a.first().unwrap_or(&Value::Null), dpi)?.unwrap_or(0.0),
            length(a.get(1).unwrap_or(&Value::Null), dpi)?,
        ),
        Value::Number(_) => {
            let n = length(&size, dpi)?;
            (n.unwrap_or(0.0), n)
        }
        Value::Object(o) => (
            length(o.get("width").unwrap_or(&Value::Null), dpi)?.unwrap_or(0.0),
            length(o.get("height").unwrap_or(&Value::Null), dpi)?,
        ),
        _ => (595.28, Some(841.89)),
    };
    let landscape = raw.props.get("orientation").and_then(Value::as_str) == Some("landscape");
    Ok(if landscape {
        (h.unwrap_or(0.0), Some(w).filter(|w| *w != 0.0), true)
    } else {
        (w, h, false)
    })
}

/// Builds a page node: the size overrides the style's width and height, percentages of page
/// paddings and of its children's heights become points against the page.
pub fn build(mut raw: RawNode) -> Result<(Node, Container), String> {
    let (width, height, landscape) = size(&raw)?;
    let mut style = flatten::flatten(&raw.style);
    style.insert("width".into(), json!(width));
    match height {
        Some(h) => style.insert("height".into(), json!(h)),
        None => style.shift_remove("height"),
    };
    let rem_base = style
        .get("fontSize")
        .and_then(Value::as_f64)
        .filter(|f| *f != 0.0)
        .unwrap_or(18.0);
    let dpi = raw
        .props
        .get("dpi")
        .and_then(Value::as_f64)
        .filter(|d| *d != 0.0)
        .unwrap_or(72.0);
    let c = Container {
        width,
        height: height.unwrap_or(f64::NAN),
        landscape,
        dpi,
        rem_base,
    };
    raw.style = Value::Object(style);
    crate::tree::node::clear_styles();
    let mut page = Node::build(raw, &c, None);

    for (i, against) in [(0, height), (1, Some(width)), (2, height), (3, Some(width))] {
        if let (Some(Dim::Pct(p)), Some(full)) = (page.style.padding[i], against) {
            Rc::make_mut(&mut page.style).padding[i] = Some(Dim::Pt(p / 100.0 * full));
        }
    }
    if let Some(h) = height {
        let pad = |i: usize| match page.style.padding[i] {
            Some(Dim::Pt(x)) => x,
            _ => 0.0,
        };
        let area = h - pad(0) - pad(2);
        for child in &mut page.children {
            if let Some(Dim::Pct(p)) = child.style.height {
                Rc::make_mut(&mut child.style).height = Some(Dim::Pt(p / 100.0 * area));
            }
        }
    }
    Ok((page, c))
}

/// The page's height and its top and bottom padding.
pub fn heights(page: &Node) -> (Option<f64>, f64, f64) {
    let pad = |i: usize| match page.style.padding[i] {
        Some(Dim::Pt(x)) => x,
        _ => 0.0,
    };
    let h = match page.style.height {
        Some(Dim::Pt(h)) => Some(h),
        _ => None,
    };
    (h, pad(0), pad(2))
}
