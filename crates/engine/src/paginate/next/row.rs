//! Rows: their children sit side by side, so a break cuts all of them at one line, and a child
//! that can neither fit nor be cut there moves the whole row.

use std::rc::Rc;

use super::fill::fill;
use super::flow::{edge_top, flagged, is_absolute, outer, with_edges};
use super::items::{FillResult, Fragment, Item, Materialize, Placed, Step, height, reach, space};
use super::overflow::{repeats, split_leaf};
use super::state::State;
use crate::tree::Node;

/// Places a row's children at the top of `height`. `None` when one cannot fit or be cut.
pub fn place(
    row: &mut Fragment,
    height_left: f64,
    page: u32,
    can_force: bool,
    above: bool,
    mx: &mut Materialize,
) -> Result<Option<FillResult>, String> {
    let mut placed = Vec::new();
    let mut remaining = Vec::new();
    for cf in row.children.iter_mut() {
        let child = cf.item.clone();
        if child.is_group() {
            let mut kids = std::mem::take(&mut cf.children);
            let inner = fill(&mut kids, height_left, page, can_force, above, mx);
            cf.children = kids;
            let inner = inner?;
            let broke = !inner.remaining.is_empty();
            if broke && inner.placed.is_empty() {
                return Ok(None);
            }
            if broke {
                let mut children = repeats(cf, &inner, above);
                children.extend(inner.remaining);
                remaining.push(Fragment {
                    item: child.clone(),
                    first: false,
                    children,
                    origin: None,
                });
            }
            placed.push(Placed {
                item: child,
                y: 0.0,
                height: if broke {
                    height_left
                } else {
                    reach(&inner.placed)
                },
                first: cf.first,
                last: !broke,
                children: Some(inner.placed),
            });
            continue;
        }
        if let Item::Leaf(l) = &*child
            && l.height > height_left
        {
            // Cut exactly at the line, or the row cannot break here.
            let Some((cur, next)) = split_leaf(&child, height_left) else {
                return Ok(None);
            };
            placed.push(Placed {
                item: cur,
                y: 0.0,
                height: height_left,
                first: cf.first,
                last: false,
                children: None,
            });
            remaining.push(Fragment {
                item: next,
                first: false,
                children: Vec::new(),
                origin: None,
            });
            continue;
        }
        let f = Fragment {
            children: Vec::new(),
            ..cf.clone()
        };
        placed.push(Placed {
            item: child,
            y: 0.0,
            height: height(&f),
            first: cf.first,
            last: true,
            children: None,
        });
    }
    Ok(Some(FillResult { placed, remaining }))
}

pub fn fit(
    st: &mut State,
    frags: &mut [Fragment],
    i: usize,
    mx: &mut Materialize,
) -> Result<Step, String> {
    let avail = st.height - st.used;
    if avail <= 0.0 {
        return Ok(Step::Decline);
    }
    let can_force = st.can_force && st.placed.is_empty();
    let above = st.content_above(frags);
    let Some(inner) = place(&mut frags[i], avail, st.page, can_force, above, mx)? else {
        return Ok(Step::Decline);
    };
    let broke = !inner.remaining.is_empty();
    // Cutting the row is a break at the page bottom, which a window forbids.
    if broke && st.height < st.forbid_until {
        return Ok(Step::Decline);
    }
    let (item, first) = (frags[i].item.clone(), frags[i].first);
    st.placed.push(Placed {
        item: item.clone(),
        y: st.used,
        height: if broke {
            avail
        } else {
            inner.placed.iter().map(|p| p.height).fold(0.0, f64::max)
        },
        first,
        last: !broke,
        children: Some(inner.placed),
    });
    if !broke {
        return Ok(Step::Continue);
    }
    let mut rest = vec![Fragment {
        item,
        first: false,
        children: inner.remaining,
        origin: None,
    }];
    rest.extend_from_slice(&frags[i + 1..]);
    Ok(st.done(rest))
}

/// A row's cells share one top. A cell its alignment pushed down carries that offset as a
/// spacer it may not be parted from.
pub fn item(mut node: Node) -> Item {
    let edge = edge_top(&node);
    let mut kept = Vec::new();
    let mut cells = Vec::new();
    let mut bottom = f64::NEG_INFINITY;
    for (i, child) in std::mem::take(&mut node.children).into_iter().enumerate() {
        if is_absolute(&child) {
            kept.push((i, child));
            continue;
        }
        let (top, h) = outer(&child);
        bottom = bottom.max(top + h);
        let item = Rc::new(flagged(child));
        let offset = top - edge;
        cells.push(if offset <= 0.001 {
            item
        } else {
            Rc::new(Item::Column {
                data: None,
                repeat: false,
                children: vec![
                    Rc::new(space(offset, false)),
                    Rc::new(Item::Penalty {
                        force: false,
                        ahead: 0.0,
                    }),
                    item,
                ],
            })
        });
    }
    let body = vec![Rc::new(Item::Row { children: cells })];
    with_edges(node, kept, edge, bottom, body)
}
