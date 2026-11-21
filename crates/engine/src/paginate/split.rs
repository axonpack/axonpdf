//! Cutting laid out content at a page break.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::layout::remainder::ids;
use crate::layout::yoga::NodeId;
use crate::style::{BOTTOM, Dim, TOP};
use crate::tree::node::TextBox;
use crate::tree::{Kind, Node, extras};

/// Boxes that cross the cut by less than this stay whole.
const SAFETY: f64 = 0.001;

/// Splits `nodes` (siblings, tops relative to their parent) at `height`: what stays above the
/// cut, and what goes to the next page (tops moved up by `height`). `content_area` is the page's
/// height inside its padding.
/// The Yoga nodes of whatever moves whole onto the finished page are added to `gone`.
pub fn split_nodes(
    height: f64,
    content_area: f64,
    nodes: Vec<Node>,
    gone: &mut Vec<NodeId>,
) -> (Vec<Node>, Vec<Node>) {
    let mut current: Vec<Node> = Vec::new();
    let mut next: Vec<Node> = Vec::new();
    let mut rest: VecDeque<Node> = nodes.into();
    let future_fixed = |rest: &VecDeque<Node>| {
        rest.iter()
            .filter(|n| n.is_fixed())
            .cloned()
            .collect::<Vec<_>>()
    };
    while let Some(mut child) = rest.pop_front() {
        let top = child.layout.top;
        let h = child.layout.height;
        if child.is_fixed() {
            current.push(child.clone());
            next.push(child);
            continue;
        }
        if height <= top {
            child.layout.top -= height;
            next.push(child);
            // When all that follows is below the cut too (or fixed), move it in one go.
            if rest.iter().all(|n| n.is_fixed() || height <= n.layout.top) {
                current.extend(future_fixed(&rest));
                next.extend(rest);
                break;
            }
            continue;
        }
        let can_wrap = child.can_wrap();
        if h > content_area && !can_wrap {
            eprintln!(
                "Node of type {} can't wrap between pages and it's bigger than available page height",
                child.kind.name()
            );
            ids(&child, gone);
            current.push(child);
            next.extend(rest);
            break;
        }
        if should_break(&child, &rest, height, &current) {
            child.layout.top -= height;
            child.props.insert("wrap".into(), true.into());
            child.props.insert("break".into(), false.into());
            current.extend(future_fixed(&rest));
            next.push(child);
            next.extend(rest);
            break;
        }
        if height + SAFETY < top + h {
            let had_children = !child.children.is_empty();
            let mark = gone.len();
            let saved = (child.style.clone(), child.props.clone(), child.layout);
            let (cur, nxt) = split(child, height, content_area, gone);
            // Nothing of a parent stays: move it whole, unless the page would be empty.
            if had_children && cur.children.is_empty() {
                // Every child went on, so the next piece is the parent once its own edges and
                // props are put back.
                gone.truncate(mark);
                let mut original = nxt;
                (original.style, original.props, original.layout) = saved;
                original.dirty = true;
                if current.is_empty() {
                    ids(&original, gone);
                    current.push(original);
                    current.extend(future_fixed(&rest));
                    next.extend(rest);
                } else {
                    let mut moved = original;
                    moved.layout.top -= height;
                    current.extend(future_fixed(&rest));
                    next.push(moved);
                    next.extend(rest);
                }
                break;
            }
            current.push(cur);
            next.push(nxt);
            continue;
        }
        ids(&child, gone);
        current.push(child);
    }
    (current, next)
}

fn should_break(child: &Node, future: &VecDeque<Node>, height: f64, previous: &[Node]) -> bool {
    if child.has("fixed") {
        return false;
    }
    let l = &child.layout;
    let crosses = height < l.top + l.height;
    let presence =
        l.top + l.height + l.margin[BOTTOM] + child.number("minPresenceAhead").unwrap_or(0.0);
    // The presence wanted ends at the furthest following sibling, if that comes sooner. It
    // passes the cut when both do; the first sibling found past it settles that.
    let mut following = future.iter().filter(|n| !n.has("fixed")).peekable();
    let end_passes = presence > height
        && (following.peek().is_none()
            || following.any(|n| n.layout.top + n.layout.height > height));
    // Breaking only helps when something else is already on this page.
    let improves = previous.iter().any(|n| !n.is_fixed());
    child.is("break") || (crosses && !child.can_wrap()) || (!crosses && improves && end_passes)
}

fn split(node: Node, height: f64, content_area: f64, gone: &mut Vec<NodeId>) -> (Node, Node) {
    if node.kind == Kind::Text {
        split_text(node, height)
    } else {
        split_view(node, height, content_area, gone)
    }
}

/// Drops the edges that fall inside the break: the bottom ones above it, the top ones below.
pub(super) fn cut_edges(n: &mut Node, side: usize) {
    // A table keeps its border on every page it runs over, closed at each cut.
    let table = n.has(extras::TABLE);
    let s = Rc::make_mut(&mut n.style);
    s.margin[side] = Some(Dim::Pt(0.0));
    s.padding[side] = Some(Dim::Pt(0.0));
    if table {
        return;
    }
    s.border_width[side] = Some(0.0);
    let corners = if side == TOP { [0, 1] } else { [2, 3] };
    for c in corners {
        s.border_radius[c] = Some(Dim::Pt(0.0));
    }
}

fn split_view(
    mut node: Node,
    height: f64,
    content_area: f64,
    gone: &mut Vec<NodeId>,
) -> (Node, Node) {
    let top = node.layout.top;
    let children = std::mem::take(&mut node.children);
    // Children still waiting as JSON lie below the cut.
    let pending = std::mem::take(&mut node.pending);
    let (cur_children, next_children) = split_nodes(height - top, content_area, children, gone);

    let mut cur = node.clone();
    cut_edges(&mut cur, BOTTOM);
    Rc::make_mut(&mut cur.style).height = Some(Dim::Pt(height - top));
    cur.children = cur_children;

    let fixed_height = node.style.height.is_some();
    let mut next = node;
    cut_edges(&mut next, TOP);
    next.layout.top = 0.0;
    next.props
        .insert("bookmark".into(), serde_json::Value::Null);
    let rest = next.layout.height - (height - top);
    if fixed_height && rest != 0.0 {
        Rc::make_mut(&mut next.style).height = Some(Dim::Pt(rest));
    }
    next.children = next_children;
    next.pending = pending;
    next.dirty = true;
    (cur, next)
}

/// Text splits between lines, keeping `orphans` lines above and `widows` below (2 each when
/// unset or 0).
pub(super) fn split_text(node: Node, height: f64) -> (Node, Node) {
    let lines = node
        .text
        .as_ref()
        .and_then(|t| t.borrow().lines.clone())
        .unwrap_or_default();
    let align_offset = node.text.as_ref().map_or(0.0, |t| t.borrow().align_offset);
    let top_of = |i: usize| -> f64 {
        if lines.is_empty() || i == 0 {
            return 0.0;
        }
        let y0 = lines[0].y;
        match lines.get(i) {
            Some(l) => l.y - y0,
            None => lines[lines.len() - 1].y - y0 + lines[lines.len() - 1].height,
        }
    };
    let at = {
        let limit = height - node.layout.top;
        (0..lines.len())
            .find(|&i| top_of(i) + lines[i].height > limit)
            .unwrap_or(lines.len())
    };
    let n = lines.len();
    let widows = node.number("widows").filter(|w| *w != 0.0).unwrap_or(2.0) as usize;
    let orphans = node.number("orphans").filter(|o| *o != 0.0).unwrap_or(2.0) as usize;
    let cut = if at == 0 {
        0
    } else if n < orphans {
        n
    } else if at < orphans || n < orphans + widows {
        0
    } else if n == orphans + widows {
        orphans
    } else if n - at < widows {
        n - widows
    } else {
        at
    };
    let current_height = top_of(cut);
    let piece = |ls: Vec<crate::text::Line>| {
        Some(Rc::new(RefCell::new(TextBox {
            lines: Some(ls),
            align_offset,
            wrapped: None,
        })))
    };

    let mut cur = node.clone();
    cut_edges(&mut cur, BOTTOM);
    cur.layout.height = current_height;
    cur.text = piece(lines[..cut].to_vec());

    // The next piece keeps the text box Yoga measures through, with the remaining lines.
    let mut next = node;
    cut_edges(&mut next, TOP);
    next.layout.top = 0.0;
    next.layout.height -= current_height;
    match &next.text {
        Some(t) => t.borrow_mut().lines = Some(lines[cut..].to_vec()),
        None => next.text = piece(lines[cut..].to_vec()),
    }
    next.dirty = true;
    (cur, next)
}
