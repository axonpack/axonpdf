//! A page's `layout` template: the nodes its function returns, with one `SLOT` node where the
//! page's content goes. The rest is chrome, drawn on every page with its space kept.

use serde_json::json;
use serde_json::value::RawValue;

use crate::style::Container;
use crate::tree::{Kind, Node, RawNode};

pub const SLOT: &str = "SLOT";

fn kind_of(json: &RawValue) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct K {
        #[serde(rename = "type")]
        kind: String,
    }
    serde_json::from_str::<K>(json.get()).ok().map(|k| k.kind)
}

/// Puts `payload` where the first `SLOT` is in `list`, counting the slots in `found` and
/// writing the way down to the payload's first node in `path`.
pub fn splice(
    list: Vec<Box<RawValue>>,
    payload: &mut Option<Vec<Box<RawValue>>>,
    found: &mut usize,
    path: &mut Vec<usize>,
) -> Vec<Box<RawValue>> {
    let mut out = Vec::new();
    for json in list {
        if kind_of(&json).as_deref() == Some(SLOT) {
            *found += 1;
            if let Some(p) = payload.take() {
                path.push(out.len());
                out.extend(p);
            }
        } else if crate::tree::has_type(json.get(), SLOT) {
            let mut node = RawNode::parse(&json);
            let (had, mut inner) = (payload.is_some(), Vec::new());
            node.children = splice(
                std::mem::take(&mut node.children),
                payload,
                found,
                &mut inner,
            );
            if had && payload.is_none() {
                path.push(out.len());
                path.extend(inner);
            }
            out.push(node.boxed());
        } else {
            out.push(json);
        }
    }
    out
}

pub fn check(found: usize) -> Result<(), String> {
    if found == 1 {
        return Ok(());
    }
    Err(format!(
        "[layout] A page layout must render its children exactly once (found {found})."
    ))
}

/// Builds a node from JSON, styled under `parent`.
pub fn node(json: serde_json::Value, c: &Container, parent: &Node) -> Node {
    let raw: RawNode = serde_json::from_value(json).expect("a node is always JSON");
    Node::build(raw, c, Some((&parent.style, false)))
}

/// Stands for the content while the room left for it is measured.
pub fn probe(c: &Container, page: &Node) -> Node {
    let style = json!({"flexGrow": 1, "flexShrink": 1, "alignSelf": "stretch"});
    node(json!({"type": "VIEW", "style": style}), c, page)
}

/// The way down to the `SLOT` node, and how many there are.
pub fn find(nodes: &[Node], path: &mut Vec<usize>, first: &mut Option<Vec<usize>>) -> usize {
    let mut count = 0;
    for (i, n) in nodes.iter().enumerate() {
        path.push(i);
        if n.kind == Kind::Other(SLOT.into()) {
            count += 1;
            first.get_or_insert_with(|| path.clone());
        } else {
            count += find(&n.children, path, first);
        }
        path.pop();
    }
    count
}

/// Replaces the node at `path` with `payload`.
pub fn put(nodes: &mut Vec<Node>, path: &[usize], payload: Vec<Node>) {
    match path {
        [i] => {
            nodes.splice(*i..*i + 1, payload);
        }
        [i, rest @ ..] => put(&mut nodes[*i].children, rest, payload),
        [] => {}
    }
}

/// The box of the node at `path` under `page`, in page coordinates: top, left, width, height.
pub fn region(page: &Node, path: &[usize]) -> (f64, f64, f64, f64) {
    let (mut n, mut top, mut left) = (page, page.layout.top, page.layout.left);
    for &i in path {
        n = &n.children[i];
        top += n.layout.top;
        left += n.layout.left;
    }
    (top, left, n.layout.width, n.layout.height)
}
