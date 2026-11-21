//! Style arrays and media queries, before any value is read.

use serde_json::{Map, Value};

use super::units::Container;

/// Merges a style or a (nested) array of styles into one object. Later entries win; `null`,
/// `false` and other non-objects are skipped, and so are keys set to `null`.
pub fn flatten(style: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    merge(style, &mut out);
    out
}

fn merge(style: &Value, out: &mut Map<String, Value>) {
    match style {
        Value::Array(items) => items.iter().for_each(|s| merge(s, out)),
        Value::Object(obj) => {
            for (k, v) in obj {
                if !v.is_null() {
                    out.insert(k.clone(), v.clone());
                }
            }
        }
        _ => {}
    }
}

/// Replaces each `@media ...` key with its styles when the query matches the page.
pub fn media(c: &Container, style: Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for (k, v) in style {
        if k.starts_with("@media") {
            if let (true, Value::Object(inner)) = (matches(c, &k), v) {
                out.extend(inner);
            }
        } else {
            out.insert(k, v);
        }
    }
    out
}

/// `@media min-width: 300 and orientation: landscape, max-height: 500`: queries joined by
/// `and` and `,`, read left to right. Sizes are plain numbers in points.
fn matches(c: &Container, query: &str) -> bool {
    let body = query.trim_start_matches('@').trim_start_matches(|ch: char| ch.is_ascii_alphabetic());
    let mut result: Option<bool> = None;
    let mut op = "and";
    for token in tokens(body) {
        match token {
            Token::Op(o) => op = o,
            Token::Query(key, value) => {
                let m = query_matches(c, &key, &value);
                result = Some(match (result, op) {
                    (None, _) => m,
                    (Some(r), ",") => r || m,
                    (Some(r), _) => r && m,
                });
            }
        }
    }
    result.unwrap_or(false)
}

enum Token {
    Op(&'static str),
    Query(String, String),
}

fn tokens(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut pending_key: Option<String> = None;
    let words = s.replace(['(', ')'], " ").replace(',', " , ").replace(':', " : ");
    let mut it = words.split_whitespace().peekable();
    while let Some(w) = it.next() {
        match w {
            "," => out.push(Token::Op(",")),
            "and" => out.push(Token::Op("and")),
            ":" => {
                if let (Some(k), Some(v)) = (pending_key.take(), it.next()) {
                    out.push(Token::Query(k, v.to_string()));
                }
            }
            _ => pending_key = Some(w.to_string()),
        }
    }
    out
}

fn query_matches(c: &Container, key: &str, value: &str) -> bool {
    let size = || -> f64 {
        let digits: String = value.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        digits.parse().unwrap_or(f64::NAN)
    };
    match key {
        "max-height" => size() >= c.height,
        "min-height" => size() <= c.height,
        "max-width" => size() >= c.width,
        "min-width" => size() <= c.width,
        "orientation" => value == if c.landscape { "landscape" } else { "portrait" },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flattens_arrays() {
        let s = flatten(&json!([{"margin": 1, "color": "red"}, null, false, [{"color": "blue", "padding": null}]]));
        assert_eq!(Value::Object(s), json!({"margin": 1, "color": "blue"}));
    }

    #[test]
    fn media_queries() {
        let c = Container { width: 400.0, height: 600.0, ..Default::default() };
        let style = flatten(&json!({
            "color": "red",
            "@media max-width: 500": {"color": "blue"},
            "@media min-width: 500": {"margin": 3},
            "@media (orientation: portrait) and (min-height: 600)": {"padding": 2},
            "@media min-width: 900, max-height: 700": {"top": 1}
        }));
        assert_eq!(Value::Object(media(&c, style)), json!({"color": "blue", "padding": 2, "top": 1}));
    }
}
