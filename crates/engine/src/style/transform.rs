//! `transform` and `transformOrigin`.

use serde_json::Value;

use super::units::{Container, Dim, V, dim, parse_float, unit, unit_str};

/// One operation, in degrees and points: `scale [x, y]`, `rotate [angle, cx, cy]`,
/// `translate [x, y]`, `skew [x, y]`, `matrix [a, b, c, d, e, f]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub op: String,
    pub values: Vec<f64>,
}

fn float(s: &str) -> f64 {
    parse_float(&Value::String(s.into())).unwrap_or(f64::NAN)
}

/// Degrees, or radians with `rad`.
fn angle(s: &str) -> f64 {
    let n = float(s);
    if s.trim_start_matches(|c: char| c == '-' || c == '.' || c.is_ascii_digit()) == "rad" {
        n.to_degrees()
    } else {
        n
    }
}

/// `"rotate(45deg) scale(2, 1)"` into operations. Anything else (an already parsed array, a
/// keyword) gives none.
pub fn parse(v: &Value) -> Option<Vec<Transform>> {
    let s = v.as_str()?.trim();
    let mut out = Vec::new();
    for part in s.split(')') {
        let part = part.trim_start_matches([' ', ',']);
        let Some((name, args)) = part.split_once('(') else { continue };
        let sep = if args.contains(',') { ',' } else { ' ' };
        let args: Vec<&str> = args.split(sep).map(str::trim).collect();
        let at = |i: usize| args.get(i).copied().unwrap_or("");
        let (op, values) = match name.trim() {
            "scale" => ("scale", vec![float(at(0)), if args.len() > 1 { float(at(1)) } else { float(at(0)) }]),
            "scaleX" => ("scale", vec![float(at(0)), 1.0]),
            "scaleY" => ("scale", vec![1.0, float(at(0))]),
            "rotate" => ("rotate", vec![angle(at(0)), float_or_zero(at(1)), float_or_zero(at(2))]),
            "translate" => ("translate", vec![float(at(0)), float_or_zero(at(1))]),
            "translateX" => ("translate", vec![float(at(0)), 0.0]),
            "translateY" => ("translate", vec![0.0, float(at(0))]),
            "skew" => ("skew", args.iter().map(|a| angle(a)).collect()),
            "skewX" => ("skew", vec![angle(at(0)), 0.0]),
            "skewY" => ("skew", vec![0.0, angle(at(0))]),
            other => (other, args.iter().map(|a| float(a)).collect()),
        };
        out.push(Transform { op: op.into(), values });
    }
    Some(out)
}

fn float_or_zero(s: &str) -> f64 {
    if s.is_empty() { 0.0 } else { float(s) }
}

/// A position keyword or length along one axis.
pub fn origin_value(c: &Container, s: &str) -> Option<Dim> {
    match s {
        "left" | "top" => Some(Dim::Pct(0.0)),
        "right" | "bottom" => Some(Dim::Pct(100.0)),
        "center" => Some(Dim::Pct(50.0)),
        _ => dim(&unit_str(c, s)),
    }
}

/// `transformOrigin: "left top"`, with a `top`/`bottom` keyword always read as the y value.
pub fn origin(c: &Container, v: &Value) -> [Option<Dim>; 2] {
    let s = match unit(c, v) {
        Some(V::Num(n)) => return [Some(Dim::Pt(n)), Some(Dim::Pct(50.0))],
        Some(V::Str(s)) => s,
        None => return [None, None],
    };
    let parts: Vec<&str> = s.split(' ').collect();
    let mut pair = match parts.as_slice() {
        [] | [""] => ["center", "center"],
        [one] => [*one, "center"],
        [x, y, ..] => [*x, *y],
    };
    if matches!(pair[0], "top" | "bottom") {
        pair.swap(0, 1);
    }
    [origin_value(c, pair[0]), origin_value(c, pair[1])]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_operations() {
        let t = parse(&json!("rotate(45deg) scale(2, 1) translate(10px 5) skewX(0.5rad) matrix(1,0,0,1,2,3)")).unwrap();
        let ops: Vec<&str> = t.iter().map(|t| t.op.as_str()).collect();
        assert_eq!(ops, ["rotate", "scale", "translate", "skew", "matrix"]);
        assert_eq!(t[0].values, [45.0, 0.0, 0.0]);
        assert_eq!(t[1].values, [2.0, 1.0]);
        assert_eq!(t[2].values, [10.0, 5.0]);
        assert!((t[3].values[0] - 0.5f64.to_degrees()).abs() < 1e-12);
        assert_eq!(t[4].values, [1.0, 0.0, 0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn origins() {
        let c = Container::default();
        assert_eq!(origin(&c, &json!("top left")), [Some(Dim::Pct(0.0)), Some(Dim::Pct(0.0))]);
        assert_eq!(origin(&c, &json!("10 20%")), [Some(Dim::Pt(10.0)), Some(Dim::Pct(20.0))]);
    }
}
