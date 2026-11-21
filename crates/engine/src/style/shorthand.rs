//! Shorthands that expand into several properties: box model, flex, borders, object position.

use serde_json::Value;

use super::expand::as_text;
use super::units::{Container, Dim, V, dim, parse_float, split_number, unit, unit_str};
use super::{BOTTOM, LEFT, RIGHT, Style, TOP, color};
use crate::display::BorderStyle;

/// `margin: "10 auto"` and friends. A value with too many parts, or a unit the box model does
/// not take, sets nothing.
pub(super) fn box_model(
    edges: &mut [Option<Dim>; 4],
    c: &Container,
    key: &str,
    base: &str,
    v: &Value,
    auto: bool,
) {
    let t = as_text(v).unwrap_or_default();
    let parts: Vec<&str> = t.split_whitespace().collect();
    let ok = |p: &&str| {
        (auto && *p == "auto")
            || p.strip_suffix('%').map_or_else(
                || {
                    split_number(p).is_some_and(|(_, u)| {
                        ["", "px", "in", "mm", "cm", "pt", "vw", "vh", "rem"].contains(&u)
                    })
                },
                |n| n.parse::<f64>().is_ok(),
            )
    };
    let side = &key[base.len()..];
    let max = match side {
        "" => 4,
        "Vertical" | "Horizontal" => 2,
        _ => 1,
    };
    if parts.is_empty() || parts.len() > max || !parts.iter().all(ok) {
        return;
    }
    let get = |i: usize| dim(&unit_str(c, parts[i]));
    let first = get(0);
    let second = parts.get(1).map_or(first, |_| get(1));
    match side {
        "" => {
            let third = parts.get(2).map_or(first, |_| get(2));
            let fourth = parts.get(3).map(|_| get(3)).unwrap_or(second);
            *edges = [first, second, third, fourth];
        }
        "Vertical" => (edges[TOP], edges[BOTTOM]) = (first, second),
        "Horizontal" => (edges[RIGHT], edges[LEFT]) = (first, second),
        "Top" => edges[TOP] = first,
        "Right" => edges[RIGHT] = first,
        "Bottom" => edges[BOTTOM] = first,
        _ => edges[LEFT] = first,
    }
}

/// `flex: n` is grow n, shrink 1, basis 0 (not `auto`); `auto`, `none` and `initial` as in CSS.
pub(super) fn flex(s: &mut Style, c: &Container, v: &Value) {
    let t = as_text(v).unwrap_or_default();
    let defaults: [&str; 3] = match t.as_str() {
        "auto" => ["1", "1", "auto"],
        "none" => ["0", "0", "auto"],
        "initial" => ["0", "1", "auto"],
        _ => ["1", "1", "0"],
    };
    let parts: Vec<&str> = if defaults[2] == "auto" {
        vec![]
    } else {
        t.split(' ').collect()
    };
    let part = |i: usize| {
        parts
            .get(i)
            .copied()
            .filter(|p| !p.is_empty())
            .unwrap_or(defaults[i])
    };
    s.flex_grow = parse_float(&Value::String(part(0).into()));
    s.flex_shrink = parse_float(&Value::String(part(1).into()));
    s.flex_basis = dim(&unit_str(c, part(2)));
}

pub(super) fn border(s: &mut Style, c: &Container, key: &str, v: &Value) {
    const SIDES: [&str; 4] = ["Top", "Right", "Bottom", "Left"];
    const CORNERS: [&str; 4] = ["TopLeft", "TopRight", "BottomRight", "BottomLeft"];
    let rest = &key["border".len()..];
    let t = as_text(v).unwrap_or_default();
    for (i, corner) in CORNERS.iter().enumerate() {
        if rest.strip_prefix(corner) == Some("Radius") {
            s.border_radius[i] = unit(c, v).as_ref().and_then(dim);
            return;
        }
    }
    // The side this key names, if any, and what follows it.
    let (sides, prop): (Vec<usize>, &str) = match SIDES.iter().position(|sd| rest.starts_with(sd)) {
        Some(i) => (vec![i], &rest[SIDES[i].len()..]),
        None => ((0..4).collect(), rest),
    };
    match prop {
        "Color" => sides
            .iter()
            .for_each(|&i| s.border_color[i] = Some(color::parse(&t))),
        "Style" => sides
            .iter()
            .for_each(|&i| s.border_style[i] = Some(border_style(&t))),
        "Width" => {
            let w = unit(c, v).and_then(|v| if let V::Num(n) = v { Some(n) } else { None });
            sides.iter().for_each(|&i| s.border_width[i] = w);
        }
        "Radius" => {
            let r = unit(c, v).as_ref().and_then(dim);
            s.border_radius = [r; 4];
        }
        "" => {
            // "width style color", e.g. "1pt solid red".
            let mut words = t
                .trim()
                .splitn(3, char::is_whitespace)
                .filter(|w| !w.is_empty());
            let (Some(w), Some(st), Some(col)) = (words.next(), words.next(), words.next()) else {
                return;
            };
            let Some(V::Num(width)) = split_number(w).map(|_| unit_str(c, w)) else {
                return;
            };
            for &i in &sides {
                s.border_width[i] = Some(width);
                s.border_style[i] = Some(border_style(st));
                s.border_color[i] = Some(color::parse(col.trim()));
            }
        }
        _ => {}
    }
}

fn border_style(s: &str) -> BorderStyle {
    match s {
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        _ => BorderStyle::Solid,
    }
}

/// `objectPosition: "left top"`; one vertical keyword centers the other axis.
pub(super) fn object_position(s: &mut Style, c: &Container, v: &Value) {
    let t = as_text(v).unwrap_or_default();
    let parts: Vec<&str> = t.split(' ').collect();
    let (x, y) = match parts.as_slice() {
        [one] if *one == "top" || *one == "bottom" => ("center", *one),
        [one] => (*one, "center"),
        [x, y, ..] => (*x, *y),
        [] => return,
    };
    s.object_position = [offset(c, x), offset(c, y)];
}

/// Position keywords as percentages.
pub(super) fn offset(c: &Container, v: &str) -> Option<Dim> {
    match v {
        "left" | "top" => Some(Dim::Pct(0.0)),
        "right" | "bottom" => Some(Dim::Pct(100.0)),
        "center" => Some(Dim::Pct(50.0)),
        _ => dim(&unit_str(c, v)),
    }
}
