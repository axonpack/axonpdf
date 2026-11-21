//! axonpdf's own nodes, turned into the standard ones the engine lays out.
//!
//! - `breakBefore: "page"` is `break`, `breakInside: "avoid"` is `wrap={false}`, and
//!   `breakAfter: "page"` puts a `break` on the next sibling.
//! - `TABLE` and `TABLE_CELL` are views, `TABLE_ROW` a view laid out as a row, and
//!   `TABLE_HEADER` a row that is `fixed`, so it repeats at the top of every page the table
//!   reaches.
//! - `PAGE_NUMBER` and `PAGE_COUNT` are text the engine fills in: the page's number, and the
//!   document's page count.
//! - `ROWS` is a view of its rows.

use serde_json::{Value, json};

use super::RawNode;

/// What a counter shows: `page` or `count`.
pub const COUNTER: &str = "__counter";

pub fn translate(mut raw: RawNode) -> RawNode {
    let p = &mut raw.props;
    if p.get("breakBefore").and_then(Value::as_str) == Some("page") {
        p.insert("break".into(), true.into());
    }
    if p.get("breakInside").and_then(Value::as_str) == Some("avoid") {
        p.insert("wrap".into(), false.into());
    }
    let row = |raw: &mut RawNode| {
        let style = std::mem::take(&mut raw.style);
        raw.style = json!([{"flexDirection": "row"}, style]);
    };
    match raw.kind.as_str() {
        "TABLE" | "TABLE_CELL" | "ROWS" => raw.kind = "VIEW".into(),
        "TABLE_ROW" => {
            raw.kind = "VIEW".into();
            row(&mut raw);
        }
        "TABLE_HEADER" => {
            raw.kind = "VIEW".into();
            raw.props.insert("fixed".into(), true.into());
            row(&mut raw);
        }
        "PAGE_NUMBER" | "PAGE_COUNT" => {
            let what = if raw.kind == "PAGE_NUMBER" {
                "page"
            } else {
                "count"
            };
            raw.kind = "TEXT".into();
            raw.props.insert(COUNTER.into(), what.into());
            raw.children.clear();
        }
        _ => {}
    }
    raw
}

/// Whether a node asks for a page break after it.
pub fn breaks_after(raw: &RawNode) -> bool {
    raw.props.get("breakAfter").and_then(Value::as_str) == Some("page")
}
