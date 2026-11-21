//! Laid out content as items. Boxes keep their first layout: a column's children go down it
//! with the blank space between them, its own edges above and below; text and childless boxes
//! are leaves that split; nodes with `render` functions wait as lazy items.

use std::rc::Rc;

use super::items::{Data, Item, ItemRef, Leaf, Split, space};
use crate::layout::yoga::{FlexDirection, PositionType};
use crate::style::{BOTTOM, TOP};
use crate::tree::{Kind, Node, extras};

pub fn is_absolute(n: &Node) -> bool {
    n.style.position == Some(PositionType::Absolute)
}

pub(super) fn is_row(n: &Node) -> bool {
    matches!(
        n.style.flex_direction,
        Some(FlexDirection::Row | FlexDirection::RowReverse)
    )
}

/// The top of the margin box, and its height.
pub(super) fn outer(n: &Node) -> (f64, f64) {
    let l = &n.layout;
    (
        l.top - l.margin[TOP],
        l.margin[TOP] + l.height + l.margin[BOTTOM],
    )
}

pub(super) fn edge_top(n: &Node) -> f64 {
    n.layout.border[TOP] + n.layout.padding[TOP]
}

/// Blank space between each flowing node and the one before it.
fn gaps(flow: &[&Node]) -> Vec<f64> {
    (0..flow.len())
        .map(|i| match i {
            0 => 0.0,
            _ => {
                let ((top, _), (prev, h)) = (outer(flow[i]), outer(flow[i - 1]));
                top - prev - h
            }
        })
        .collect()
}

/// The whole content of a page: one column with no box of its own.
pub fn root(nodes: Vec<Node>) -> Item {
    Item::Column {
        data: None,
        repeat: false,
        children: items_of(nodes),
    }
}

pub fn leaf(node: Node) -> Item {
    let split = if node.can_wrap() {
        Split::Node
    } else {
        Split::No
    };
    Item::Leaf(Leaf {
        height: outer(&node).1,
        repeat: false,
        split,
        data: Some(data(node)),
    })
}

/// A static absolutely placed node.
pub fn marker(node: Node) -> Item {
    Item::Leaf(Leaf {
        height: 0.0,
        repeat: false,
        split: Split::No,
        data: Some(data(node)),
    })
}

fn data(node: Node) -> Rc<Data> {
    Rc::new(Data {
        node,
        kept: Vec::new(),
    })
}

/// A node as an item, without the flags its place in a flow gives it.
pub fn to_item(node: Node) -> Item {
    if is_absolute(&node) {
        return if node.dynamic {
            Item::Lazy {
                node: Box::new(node),
                repeat: false,
                absolute: true,
            }
        } else {
            marker(node)
        };
    }
    let renders = node.render.is_some() || node.has(extras::COUNTER);
    if node.dynamic && (renders || is_row(&node) || !node.can_wrap()) {
        return Item::Lazy {
            node: Box::new(node),
            repeat: false,
            absolute: false,
        };
    }
    let flowing = node.children.iter().any(|c| !is_absolute(c));
    if node.kind == Kind::Text || !flowing || !node.can_wrap() {
        return leaf(node);
    }
    if is_row(&node) {
        super::row::item(node)
    } else {
        column(node)
    }
}

/// A node entering a flow, with its flags: `fixed` repeats it on every page.
pub(super) fn flagged(node: Node) -> Item {
    let fixed = node.is_fixed();
    let mut item = to_item(node);
    if fixed {
        item.set_repeat();
    }
    item
}

/// Children down a column: a forced break before a `break` node, the space before each, and a
/// window after a `minPresenceAhead` node where no break may fall.
///
/// A static absolutely placed child takes no room and stays out of the spacing. A `fixed` one that
/// comes after other children is moved first, so it is on every page, and stays out too.
fn items_of(mut children: Vec<Node>) -> Vec<ItemRef> {
    let pinned = |c: &Node| is_absolute(c) && c.is_fixed();
    let lead = children.iter().take_while(|c| pinned(c)).count();
    let (moved, rest): (Vec<Node>, Vec<Node>) =
        children.split_off(lead).into_iter().partition(pinned);
    let children: Vec<(Node, bool)> = (children.into_iter().map(|c| (c, false)))
        .chain(moved.into_iter().map(|c| (c, true)))
        .chain(rest.into_iter().map(|c| (c, false)))
        .collect();
    let spaced = |c: &Node, moved: bool| !moved && (!is_absolute(c) || c.dynamic);
    let flow: Vec<&Node> = children
        .iter()
        .filter(|(c, m)| spaced(c, *m))
        .map(|(c, _)| c)
        .collect();
    let mut gap = gaps(&flow).into_iter();
    let mut index = 0;
    let mut out = Vec::new();
    for (child, moved) in children {
        if !spaced(&child, moved) {
            out.push(Rc::new(flagged(child)));
            continue;
        }
        let before = gap.next().unwrap_or(0.0);
        let breaks = child.is("break") && index > 0;
        let presence = child.number("minPresenceAhead").filter(|p| *p > 0.0);
        index += 1;
        if breaks {
            out.push(Rc::new(Item::Penalty {
                force: true,
                ahead: 0.0,
            }));
        }
        if before > 0.0 {
            out.push(Rc::new(space(before, true)));
        }
        out.push(Rc::new(flagged(child)));
        if let Some(ahead) = presence {
            out.push(Rc::new(Item::Penalty {
                force: false,
                ahead,
            }));
        }
    }
    out
}

/// A box's edges as spacers above and below its body, which carry over a break.
pub(super) fn with_edges(
    node: Node,
    kept: Vec<(usize, Node)>,
    top: f64,
    bottom: f64,
    body: Vec<ItemRef>,
) -> Item {
    let l = node.layout;
    let (above, below) = (l.margin[TOP] + top, l.margin[BOTTOM] + l.height - bottom);
    let mut children = Vec::new();
    if above > 0.0 {
        children.push(Rc::new(space(above, false)));
    }
    children.extend(body);
    if below > 0.0 {
        children.push(Rc::new(space(below, false)));
    }
    Item::Column {
        data: Some(Rc::new(Data { node, kept })),
        repeat: false,
        children,
    }
}

fn column(mut node: Node) -> Item {
    let flow: Vec<&Node> = (node.children.iter())
        .filter(|c| !is_absolute(c) || c.dynamic)
        .collect();
    // Children that overlap wrapped into columns of their own: there is no line to break at.
    if gaps(&flow).iter().any(|g| *g < -0.001) {
        return Item::Leaf(Leaf {
            height: outer(&node).1,
            repeat: false,
            split: Split::No,
            data: Some(data(node)),
        });
    }
    let top = flow.first().map_or(edge_top(&node), |f| outer(f).0);
    let bottom = flow.last().map_or(node.layout.height, |f| {
        let (t, h) = outer(f);
        t + h
    });
    let children = std::mem::take(&mut node.children);
    with_edges(node, Vec::new(), top, bottom, items_of(children))
}
