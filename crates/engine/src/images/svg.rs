//! SVG files used as images: the XML into the tree shape the `Svg` component sends (types in
//! upper case, attributes in camelCase), so the SVG drawing code draws both. Elements it
//! cannot draw (scripts, filters, masks, patterns, `use`, animation) are left out.

use serde_json::{Map, Value, json};

use super::ImageError;

/// Whether the bytes look like an SVG document.
pub fn is_svg(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
    let s = s.trim_start_matches('\u{FEFF}').trim_start();
    s.starts_with("<?xml") || s.starts_with("<svg")
}

/// The `SVG` node and its size in points (`width`/`height`, else the viewBox).
pub fn parse(bytes: &[u8]) -> Result<(Value, f32, f32), ImageError> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| ImageError::Invalid("SVG is not UTF-8".into()))?;
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let doc = roxmltree::Document::parse_with_options(text, options)
        .map_err(|e| ImageError::Invalid(e.to_string()))?;
    let root = doc.root_element();
    if !root.tag_name().name().eq_ignore_ascii_case("svg") {
        return Err(ImageError::Invalid("no <svg> root".into()));
    }
    let tree = node(root).unwrap_or(Value::Null);
    let view = root.attribute("viewBox").map(|v| {
        v.split(|c: char| c == ',' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.parse::<f32>().ok())
            .collect::<Vec<_>>()
    });
    let view = view.filter(|v| v.len() == 4);
    let width = root
        .attribute("width")
        .and_then(length)
        .or(view.as_ref().map(|v| v[2]))
        .unwrap_or(0.0);
    let height = root
        .attribute("height")
        .and_then(length)
        .or(view.as_ref().map(|v| v[3]))
        .unwrap_or(0.0);
    Ok((tree, width, height))
}

/// A length with an optional absolute unit, in points.
fn length(s: &str) -> Option<f32> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());
    let n: f32 = s[..split].parse().ok()?;
    let per = match &s[split..] {
        "" | "pt" => 1.0,
        "px" => 72.0 / 96.0,
        "in" => 72.0,
        "cm" => 72.0 / 2.54,
        "mm" => 72.0 / 25.4,
        _ => return None,
    };
    Some(n * per)
}

fn kind(tag: &str) -> Option<&'static str> {
    Some(match tag.to_ascii_lowercase().as_str() {
        "svg" => "SVG",
        "g" => "G",
        "path" => "PATH",
        "rect" => "RECT",
        "circle" => "CIRCLE",
        "ellipse" => "ELLIPSE",
        "line" => "LINE",
        "polyline" => "POLYLINE",
        "polygon" => "POLYGON",
        "text" => "TEXT",
        "tspan" => "TSPAN",
        "defs" => "DEFS",
        "clippath" => "CLIP_PATH",
        "lineargradient" => "LINEAR_GRADIENT",
        "radialgradient" => "RADIAL_GRADIENT",
        "marker" => "MARKER",
        "stop" => "STOP",
        "image" => "IMAGE",
        _ => return None,
    })
}

fn node(n: roxmltree::Node) -> Option<Value> {
    let kind = kind(n.tag_name().name())?;
    let mut props = Map::new();
    for a in n.attributes() {
        if a.name() != "style" {
            props.insert(camel(a.name()), Value::String(a.value().into()));
        }
    }
    // Declarations in `style` win over attributes.
    for decl in n.attribute("style").unwrap_or("").split(';') {
        if let Some((k, v)) = decl.split_once(':') {
            props.insert(camel(k.trim()), Value::String(v.trim().into()));
        }
    }
    let text = matches!(kind, "TEXT" | "TSPAN");
    let children: Vec<Value> = n
        .children()
        .filter_map(|c| {
            if c.is_element() {
                return node(c);
            }
            let t = c.text().filter(|_| text && c.is_text())?;
            let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
            (!t.is_empty()).then(|| json!({"type": "TEXT_INSTANCE", "value": t}))
        })
        .collect();
    Some(json!({"type": kind, "props": props, "children": children}))
}

/// `stroke-width` to `strokeWidth`, `xlink:href` to `xlinkHref`.
fn camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut up = false;
    for c in name.chars() {
        if c == '-' || c == ':' {
            up = true;
        } else if up {
            out.extend(c.to_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}
