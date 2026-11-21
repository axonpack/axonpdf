//! axonpdf's own nodes, turned into the standard ones the engine lays out.
//!
//! - `breakBefore: "page"` is `break`, `breakInside: "avoid"` is `wrap={false}`, and
//!   `breakAfter: "page"` puts a `break` on the next sibling.
//! - `TABLE` and `TABLE_CELL` are views (a table's border is drawn on every page it reaches), `TABLE_ROW` a view laid out as a row, and
//!   `TABLE_HEADER` a row that is `fixed`, so it repeats at the top of every page the table
//!   reaches.
//! - `PAGE_NUMBER` and `PAGE_COUNT` are text the engine fills in: the page's number, and the
//!   document's page count.
//! - `ROWS` is a view of its rows.
//! - Table borders that meet collapse into one.

use serde_json::{Value, json};

use std::rc::Rc;

use super::{Node, RawNode};
use crate::style::{BOTTOM, LEFT, RIGHT, TOP};

/// Marks a table, whose border is drawn on every page it reaches.
pub const TABLE: &str = "__table";

/// Marks a table row, whose cells' borders collapse.
pub const TABLE_ROW: &str = "__table_row";

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
        "TABLE" => {
            raw.kind = "VIEW".into();
            raw.props.insert(TABLE.into(), true.into());
        }
        "TABLE_CELL" | "ROWS" => raw.kind = "VIEW".into(),
        "TABLE_ROW" => {
            raw.kind = "VIEW".into();
            raw.props.insert(TABLE_ROW.into(), true.into());
            row(&mut raw);
        }
        "TABLE_HEADER" => {
            raw.kind = "VIEW".into();
            raw.props.insert(TABLE_ROW.into(), true.into());
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

/// Borders that meet collapse into one, the wider: between rows of a table (and the table's
/// top edge and its first row), and between cells of a row (and the row's left edge and its
/// first cell). `child` is about to join `parent` after its last child.
pub fn collapse(parent: &mut Node, child: &mut Node) {
    // Rows run down a table, cells across a row: top meets bottom, left meets right.
    let (near, far) = if parent.has(TABLE) {
        (TOP, BOTTOM)
    } else if parent.has(TABLE_ROW) {
        (LEFT, RIGHT)
    } else {
        return;
    };
    let width = |n: &Node, side: usize| n.style.border_width[side].unwrap_or(0.0);
    let theirs = match parent.children.last_mut() {
        Some(prev) => (width(prev, far), Some(prev)),
        None => (width(parent, near), None),
    };
    let mine = width(child, near);
    if mine == 0.0 || theirs.0 == 0.0 {
        return;
    }
    if theirs.0 >= mine {
        Rc::make_mut(&mut child.style).border_width[near] = Some(0.0);
    } else {
        match theirs.1 {
            Some(prev) => Rc::make_mut(&mut prev.style).border_width[far] = Some(0.0),
            None => Rc::make_mut(&mut parent.style).border_width[near] = Some(0.0),
        }
    }
}
