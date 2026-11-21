//! Nodes back from placements: what a page holds, its split boxes pinned to the height they
//! took, and the leaf splits the flow asks for.

use std::cell::RefCell;
use std::rc::Rc;

use super::flow::is_absolute;
use super::items::Placed;
use crate::paginate::split::{cut_edges, split_text};
use crate::style::{BOTTOM, Dim, TOP};
use crate::tree::{Kind, Node, extras};

/// The nodes a page's placements stand for, in order.
pub fn nodes(placed: &[Placed]) -> Vec<Node> {
    let mut out = Vec::new();
    placed.iter().for_each(|p| rebuild(p, &mut out));
    out
}

fn rebuild(p: &Placed, out: &mut Vec<Node>) {
    let Some(data) = p.item.data() else {
        p.children.iter().flatten().for_each(|c| rebuild(c, out));
        return;
    };
    let mut node = data.node.clone();
    let Some(children) = &p.children else {
        out.push(node);
        return;
    };
    if is_absolute(&node) {
        out.push(node);
        return;
    }
    let mut kids = Vec::new();
    children.iter().for_each(|c| rebuild(c, &mut kids));
    node.dirty = true;
    if p.first && p.last {
        // Whole: the children it had outside the flow go back where they were.
        for (i, n) in &data.kept {
            kids.insert((*i).min(kids.len()), n.clone());
        }
        node.children = kids;
        out.push(node);
        return;
    }
    // A part that continues takes the rest of the page; the last part grows with its children.
    let m = node.layout.margin;
    let height =
        p.height - if p.first { m[TOP] } else { 0.0 } - if p.last { m[BOTTOM] } else { 0.0 };
    if !p.last {
        Rc::make_mut(&mut node.style).height = Some(Dim::Pt(height));
        cut_edges(&mut node, BOTTOM);
    }
    // Its parts keep what was rendered for them: the whole output again would fill each part.
    node.render = None;
    if !p.first {
        cut_edges(&mut node, TOP);
        node.props
            .insert("bookmark".into(), serde_json::Value::Null);
    }
    node.children = kids;
    out.push(node);
}

/// Splits a leaf so its first part is `avail` high inside its margins.
pub fn split(node: &Node, avail: f64) -> Option<(Node, Node)> {
    if avail <= 0.0 {
        return None;
    }
    let (mut cur, mut next) = if node.kind == Kind::Text {
        let mut n = node.clone();
        // The lines are shared with the node it came from, which may be placed again.
        n.text = n
            .text
            .as_ref()
            .map(|t| Rc::new(RefCell::new(t.borrow().clone())));
        let top = n.layout.top;
        let (cur, next) = split_text(n, avail + top);
        let lines = |n: &Node| {
            n.text
                .as_ref()
                .is_some_and(|t| t.borrow().lines.as_ref().is_some_and(|l| !l.is_empty()))
        };
        if !lines(&cur) || !lines(&next) {
            return None;
        }
        (cur, next)
    } else {
        if avail >= node.layout.height {
            return None;
        }
        let rest = node.layout.height - avail;
        let (mut cur, mut next) = (node.clone(), node.clone());
        Rc::make_mut(&mut cur.style).height = Some(Dim::Pt(avail));
        cut_edges(&mut cur, BOTTOM);
        cur.layout.height = avail;
        Rc::make_mut(&mut next.style).height = Some(Dim::Pt(rest));
        cut_edges(&mut next, TOP);
        (next.layout.top, next.layout.height) = (0.0, rest);
        (cur, next)
    };
    for (n, side) in [(&mut cur, BOTTOM), (&mut next, TOP)] {
        n.render = None;
        n.props.shift_remove(extras::COUNTER);
        let l = &mut n.layout;
        (l.margin[side], l.padding[side], l.border[side]) = (0.0, 0.0, 0.0);
    }
    Some((cur, next))
}
