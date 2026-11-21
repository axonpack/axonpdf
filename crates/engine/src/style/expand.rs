//! One style key at a time: shorthands expanded, units resolved, values typed.

use serde_json::{Map, Value};

use super::shorthand::{border, box_model, flex, object_position, offset};
use super::units::{Container, V, dim, parse_float, unit, unit_str};
use super::{BOTTOM, LEFT, RIGHT, Style, TOP, color};
use crate::layout::yoga::{Align, FlexDirection, Justify, Overflow, PositionType, Wrap};

pub fn apply(s: &mut Style, c: &Container, key: &str, v: &Value, flat: &Map<String, Value>) {
    let len = || unit(c, v).as_ref().and_then(dim);
    let num = || unit(c, v).and_then(|v| if let V::Num(n) = v { Some(n) } else { None });
    let text = || as_text(v);
    match key {
        "display" => s.display_none = text().as_deref() == Some("none"),
        "position" => {
            s.position = match text().as_deref() {
                Some("absolute") => Some(PositionType::Absolute),
                Some("relative") => Some(PositionType::Relative),
                Some("static") => Some(PositionType::Static),
                _ => None,
            }
        }
        "top" => s.inset[TOP] = len(),
        "right" => s.inset[RIGHT] = len(),
        "bottom" => s.inset[BOTTOM] = len(),
        "left" => s.inset[LEFT] = len(),
        "width" => s.width = len(),
        "height" => s.height = len(),
        "minWidth" => s.min_width = len(),
        "minHeight" => s.min_height = len(),
        "maxWidth" => s.max_width = len(),
        "maxHeight" => s.max_height = len(),
        "margin" | "marginVertical" | "marginHorizontal" | "marginTop" | "marginRight"
        | "marginBottom" | "marginLeft" => box_model(&mut s.margin, c, key, "margin", v, true),
        "padding" | "paddingVertical" | "paddingHorizontal" | "paddingTop" | "paddingRight"
        | "paddingBottom" | "paddingLeft" => box_model(&mut s.padding, c, key, "padding", v, false),
        "flexDirection" => {
            s.flex_direction = Some(match text().as_deref() {
                Some("row") => FlexDirection::Row,
                Some("row-reverse") => FlexDirection::RowReverse,
                Some("column-reverse") => FlexDirection::ColumnReverse,
                _ => FlexDirection::Column,
            })
        }
        "flexWrap" => {
            s.flex_wrap = Some(match text().as_deref() {
                Some("wrap") => Wrap::Wrap,
                Some("wrap-reverse") => Wrap::WrapReverse,
                _ => Wrap::NoWrap,
            })
        }
        "flexGrow" => s.flex_grow = parse_float(v),
        "flexShrink" => s.flex_shrink = parse_float(v),
        "flexBasis" => s.flex_basis = len(),
        "flex" => flex(s, c, v),
        "alignItems" => s.align_items = text().as_deref().and_then(align),
        "alignSelf" => s.align_self = text().as_deref().and_then(align),
        "alignContent" => s.align_content = text().as_deref().and_then(align),
        "justifyContent" => {
            s.justify_content = Some(match text().as_deref() {
                Some("center") => Justify::Center,
                Some("flex-end") => Justify::FlexEnd,
                Some("space-between") => Justify::SpaceBetween,
                Some("space-around") => Justify::SpaceAround,
                Some("space-evenly") => Justify::SpaceEvenly,
                _ => Justify::FlexStart,
            })
        }
        "gap" => {
            let t = text().unwrap_or_default();
            let mut parts = t.split(' ');
            let row = parts.next().unwrap_or("");
            let col = parts.next().unwrap_or(row);
            s.row_gap = dim(&unit_str(c, row));
            s.column_gap = dim(&unit_str(c, col));
        }
        "rowGap" => s.row_gap = len(),
        "columnGap" => s.column_gap = len(),
        "aspectRatio" => s.aspect_ratio = parse_float(v),
        "overflow" => {
            s.overflow = match text().as_deref() {
                Some("hidden") => Some(Overflow::Hidden),
                Some("scroll") => Some(Overflow::Scroll),
                Some(_) => Some(Overflow::Visible),
                None => None,
            }
        }
        "zIndex" => s.z_index = parse_float(v),
        "float" => s.float = text(),

        "backgroundColor" => s.background_color = text().map(|t| color::parse(&t)),
        "opacity" => s.opacity = parse_float(v),
        "transform" => s.transform = Some(v.clone()),
        "transformOrigin" => s.transform_origin = Some(v.clone()),
        "objectFit" => s.object_fit = text(),
        "objectPosition" => object_position(s, c, v),
        "objectPositionX" => s.object_position[0] = offset(c, &text().unwrap_or_default()),
        "objectPositionY" => s.object_position[1] = offset(c, &text().unwrap_or_default()),
        k if k.starts_with("border") => border(s, c, k, v),

        "color" => s.color = text().map(|t| color::parse(&t)),
        "fontFamily" => {
            s.font_family = match v {
                Value::String(f) => Some(vec![f.clone()]),
                Value::Array(a) => Some(a.iter().filter_map(as_text).collect()),
                _ => None,
            }
        }
        "fontSize" => s.font_size = num(),
        "fontStyle" => s.font_style = text(),
        "fontWeight" => s.font_weight = Some(font_weight(v)),
        "letterSpacing" => s.letter_spacing = num(),
        "lineHeight" => s.line_height = line_height(c, v, flat),
        "textAlign" => s.text_align = text(),
        "textDecoration" => s.text_decoration = text(),
        "textDecorationColor" => s.text_decoration_color = text().map(|t| color::parse(&t)),
        "textDecorationStyle" => s.text_decoration_style = text(),
        "textTransform" => s.text_transform = text(),
        "textIndent" => s.text_indent = parse_float(v),
        "maxLines" => s.max_lines = parse_float(v),
        "textOverflow" => s.text_overflow = text(),
        "verticalAlign" => s.vertical_align = text(),
        "direction" => s.direction = text(),
        "visibility" => s.visibility = text(),
        "wordSpacing" => s.word_spacing = num(),
        _ => {}
    }
}

pub(super) fn as_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn align(v: &str) -> Option<Align> {
    Some(match v {
        "flex-start" => Align::FlexStart,
        "center" => Align::Center,
        "flex-end" => Align::FlexEnd,
        "stretch" => Align::Stretch,
        "baseline" => Align::Baseline,
        "space-between" => Align::SpaceBetween,
        "space-around" => Align::SpaceAround,
        "space-evenly" => Align::SpaceEvenly,
        _ => return None,
    })
}

fn font_weight(v: &Value) -> u16 {
    const NAMES: [(&str, u16); 14] = [
        ("thin", 100),
        ("hairline", 100),
        ("ultralight", 200),
        ("extralight", 200),
        ("light", 300),
        ("normal", 400),
        ("medium", 500),
        ("semibold", 600),
        ("demibold", 600),
        ("bold", 700),
        ("ultrabold", 800),
        ("extrabold", 800),
        ("heavy", 900),
        ("black", 900),
    ];
    match v {
        Value::Number(n) => n.as_f64().map_or(400, |n| n as u16),
        Value::String(s) if s.is_empty() => 400,
        Value::String(s) => {
            let l = s.to_lowercase();
            NAMES
                .iter()
                .find(|(n, _)| *n == l)
                .map_or_else(|| parse_float(v).map_or(400, |n| n as u16), |w| w.1)
        }
        _ => 400,
    }
}

/// Unitless and percent line heights scale the font size this same style sets (18 when it sets
/// none, even if a parent sets one).
fn line_height(c: &Container, v: &Value, flat: &Map<String, Value>) -> Option<f64> {
    let font_size = flat
        .get("fontSize")
        .filter(|f| super::super::tree::truthy(f))
        .cloned();
    let font_size = match unit(c, &font_size.unwrap_or(Value::from(18))) {
        Some(V::Num(n)) => n,
        _ => f64::NAN,
    };
    let lh = unit(c, v)?;
    if let V::Str(s) = &lh {
        return super::units::percent(s).map(|p| p / 100.0 * font_size);
    }
    let V::Num(n) = lh else { return None };
    let unitless = match v {
        Value::Number(_) => true,
        Value::String(s) => s.trim().parse::<f64>().is_ok(),
        _ => false,
    };
    Some(if unitless { n * font_size } else { n })
}
