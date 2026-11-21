//! Outline entries for nodes with a `bookmark`, each under its nearest bookmarked ancestor.

use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::Value;

use crate::display::{Bookmark, Op};
use crate::tree::Node;

/// Where each bookmark went, across pages: its reference to the entry made for it last.
#[derive(Default)]
pub struct Outline {
    made: RefCell<(u32, HashMap<u64, u32>)>,
}

impl Outline {
    /// Adds `n`'s outline entry. The position is the node's box offset within its parent, as
    /// the components place it, unless the bookmark gives `left` and `top`.
    pub fn add(&self, ops: &mut Vec<Op>, n: &Node) {
        let Some(b) = n.props.get("bookmark").filter(|b| b.is_object()) else {
            return;
        };
        let num = |k: &str| b.get(k).and_then(Value::as_f64).filter(|v| *v != 0.0);
        let mut made = self.made.borrow_mut();
        let id = made.0;
        made.0 += 1;
        let parent = b
            .get("parent")
            .and_then(Value::as_u64)
            .and_then(|p| made.1.get(&p).copied());
        if let Some(r) = b.get("ref").and_then(Value::as_u64) {
            made.1.insert(r, id);
        }
        ops.push(Op::Bookmark(Bookmark {
            id,
            parent,
            title: b
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            left: num("left").unwrap_or(n.layout.left) as f32,
            top: num("top").unwrap_or(n.layout.top) as f32,
            zoom: num("zoom").map(|z| z as f32),
            fit: b.get("fit").and_then(Value::as_bool).unwrap_or(false),
            expanded: b.get("expanded").and_then(Value::as_bool).unwrap_or(false),
        }));
    }
}

thread_local! {
    /// Bookmark references, unique for as long as the engine runs.
    static NEXT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Gives every `bookmark` under `n` a reference and its nearest bookmarked ancestor's; a
/// string bookmark becomes `{title}`.
pub fn number(n: &mut Node, parent: Option<u64>) {
    let mut parent = parent;
    if let Some(b) = n
        .props
        .get_mut("bookmark")
        .filter(|b| super::super::tree::truthy(b))
    {
        if let Value::String(title) = b {
            *b = serde_json::json!({"title": title.clone(), "fit": false, "expanded": false});
        }
        if let Value::Object(o) = b {
            let r = NEXT.with(|c| c.replace(c.get() + 1));
            o.insert("ref".into(), r.into());
            if let Some(p) = parent {
                o.insert("parent".into(), p.into());
            }
            parent = Some(r);
        }
    }
    for c in &mut n.children {
        number(c, parent);
    }
}
