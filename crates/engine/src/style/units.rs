//! Units: every length a style may carry, turned into points.

use serde_json::Value;

/// What lengths resolve against: the page box, its `dpi` and the `rem` base.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Container {
    pub width: f64,
    pub height: f64,
    pub landscape: bool,
    pub dpi: f64,
    /// The page's own `fontSize`, else 18.
    pub rem_base: f64,
}

impl Default for Container {
    fn default() -> Self {
        Self { width: 0.0, height: 0.0, landscape: false, dpi: 72.0, rem_base: 18.0 }
    }
}

/// A value after unit resolution: points, or a string kept as written (`"auto"`, `"50%"`).
#[derive(Clone, Debug, PartialEq)]
pub enum V {
    Num(f64),
    Str(String),
}

/// A length for a property that may be `auto`, a percentage, or points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dim {
    Auto,
    Pt(f64),
    Pct(f64),
}

/// Splits `"12.5mm"` into the number and its unit, the way the style parser accepts it: an
/// optional minus, digits with at most one dot and a digit last, then an optional unit.
pub fn split_number(s: &str) -> Option<(f64, &str)> {
    let digits_end = s
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || (c == '-' && i == 0)))
        .map_or(s.len(), |(i, _)| i);
    let (num, unit) = s.split_at(digits_end);
    let body = num.strip_prefix('-').unwrap_or(num);
    let valid = !body.is_empty()
        && body.matches('.').count() <= 1
        && body.ends_with(|c: char| c.is_ascii_digit());
    valid.then(|| num.parse().ok().map(|n: f64| (n, unit))).flatten()
}

/// Turns a style value into points against `c`. Unknown strings pass through unchanged.
pub fn unit(c: &Container, v: &Value) -> Option<V> {
    match v {
        Value::Number(n) => n.as_f64().map(V::Num),
        Value::String(s) => Some(unit_str(c, s)),
        _ => None,
    }
}

pub fn unit_str(c: &Container, s: &str) -> V {
    let Some((n, u)) = split_number(s) else {
        return V::Str(s.to_string());
    };
    V::Num(match u {
        "" | "pt" => n,
        "rem" => n * c.rem_base,
        "in" => n * 72.0,
        "mm" => n * (72.0 / 25.4),
        "cm" => n * (72.0 / 2.54),
        "vh" => n * (c.height / 100.0),
        "vw" => n * (c.width / 100.0),
        "px" => js_round(n * (72.0 / c.dpi)),
        _ => return V::Str(s.to_string()),
    })
}

/// `Math.round`: halves go up.
pub fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// The first `<number>%` anywhere in the text, as the percentage number (`"50%"` is 50).
pub fn percent(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    for (i, _) in s.match_indices('%') {
        // Walk back over `\d+\.?\d*`, then an optional minus.
        let mut j = i;
        while j > 0 && b[j - 1].is_ascii_digit() {
            j -= 1;
        }
        if j > 0 && b[j - 1] == b'.' {
            let dot = j - 1;
            let mut k = dot;
            while k > 0 && b[k - 1].is_ascii_digit() {
                k -= 1;
            }
            if k < dot {
                j = k;
            }
        }
        if j == i || !b[j].is_ascii_digit() {
            continue;
        }
        if j > 0 && b[j - 1] == b'-' {
            j -= 1;
        }
        return s[j..i].parse().ok();
    }
    None
}

/// A resolved value as a length: points, `auto`, or a percentage.
pub fn dim(v: &V) -> Option<Dim> {
    match v {
        V::Num(n) => Some(Dim::Pt(*n)),
        V::Str(s) if s == "auto" => Some(Dim::Auto),
        V::Str(s) => percent(s).map(Dim::Pct),
    }
}

/// `parseFloat` of a JSON value: a leading number in a string, numbers as they are.
pub fn parse_float(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => {
            let s = s.trim_start();
            let end = s
                .char_indices()
                .find(|&(i, c)| {
                    !(c.is_ascii_digit() || c == '.' || ((c == '-' || c == '+') && i == 0))
                })
                .map_or(s.len(), |(i, _)| i);
            // Shrink until it parses ("1.2.3" reads as 1.2).
            (1..=end).rev().find_map(|e| s[..e].parse().ok())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn c() -> Container {
        Container { width: 600.0, height: 800.0, dpi: 144.0, ..Default::default() }
    }

    #[test]
    fn units() {
        let n = |v| match unit(&c(), &v) {
            Some(V::Num(n)) => n,
            other => panic!("{other:?}"),
        };
        assert_eq!(n(json!(12)), 12.0);
        assert_eq!(n(json!("12")), 12.0);
        assert_eq!(n(json!("1in")), 72.0);
        assert_eq!(n(json!("2.54cm")), 72.0);
        assert_eq!(n(json!("25.4mm")), 72.0);
        assert_eq!(n(json!("10vw")), 60.0);
        assert_eq!(n(json!("10vh")), 80.0);
        assert_eq!(n(json!("2rem")), 36.0);
        assert_eq!(n(json!("15px")), 8.0); // round(7.5)
        assert_eq!(n(json!(".5")), 0.5);
        assert_eq!(n(json!("-3pt")), -3.0);
        assert_eq!(unit(&c(), &json!("50%")), Some(V::Str("50%".into())));
        assert_eq!(unit(&c(), &json!("10.")), Some(V::Str("10.".into())));
        assert_eq!(unit(&c(), &json!("auto")), Some(V::Str("auto".into())));
    }

    #[test]
    fn percents() {
        assert_eq!(percent("50%"), Some(50.0));
        assert_eq!(percent("-12.5%"), Some(-12.5));
        assert_eq!(percent("calc(20%)"), Some(20.0));
        assert_eq!(percent("50"), None);
        assert_eq!(dim(&V::Str("auto".into())), Some(Dim::Auto));
    }

    #[test]
    fn parse_floats() {
        assert_eq!(parse_float(&json!("1.5abc")), Some(1.5));
        assert_eq!(parse_float(&json!("abc")), None);
        assert_eq!(parse_float(&json!(2)), Some(2.0));
    }
}
