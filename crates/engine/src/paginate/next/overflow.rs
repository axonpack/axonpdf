//! When an item does not fit: split it, go back to the last legal break, leave it for the next
//! page, or, at the top of an empty page where waiting cannot help, place it anyway.

use std::rc::Rc;

use super::items::{
    FillResult, Fragment, Item, ItemRef, Materialize, Placed, Split, Step, fully_placed, height,
    space, to_fragments,
};
use super::state::State;
use super::{build, flow, row};

pub fn overflow(
    st: &mut State,
    frags: &mut Vec<Fragment>,
    i: usize,
    mx: &mut Materialize,
) -> Result<Step, String> {
    let item = frags[i].item.clone();
    // A split is a break at the page bottom, which a window forbids.
    if st.height >= st.forbid_until
        && let Some((cur, next)) = split_leaf(&item, st.height - st.used)
    {
        let h = height(&Fragment {
            item: cur.clone(),
            first: true,
            children: Vec::new(),
            origin: None,
        });
        st.placed.push(Placed {
            item: cur,
            y: st.used,
            height: h,
            first: frags[i].first,
            last: false,
            children: None,
        });
        let mut rest = vec![Fragment {
            item: next,
            first: false,
            children: Vec::new(),
            origin: None,
        }];
        rest.extend_from_slice(&frags[i + 1..]);
        return Ok(st.done(rest));
    }
    if let Some((at, keep)) = st.best
        && at <= i
    {
        st.placed.truncate(keep);
        let from = st.restore(frags).unwrap_or(at);
        return Ok(st.done(frags[from..].to_vec()));
    }
    // With no legal break here, forcing is only right at the top of a page; anywhere else
    // everything goes back to be tried on the next page.
    if !st.can_force {
        st.restore(frags);
        return Ok(Step::Done(FillResult {
            placed: Vec::new(),
            remaining: frags.clone(),
        }));
    }
    if !st.placed.is_empty() {
        let from = st.restore(frags).unwrap_or(i);
        return Ok(st.done(frags[from..].to_vec()));
    }
    if let Some(from) = st.restore(frags) {
        return Ok(Step::Done(FillResult {
            placed: Vec::new(),
            remaining: frags[from..].to_vec(),
        }));
    }
    let h = height(&frags[i]);
    eprintln!(
        "An item of height {h} is taller than the {} the page has for it; it is placed on a page of its own.",
        st.height
    );
    let children = if matches!(&*item, Item::Row { .. }) {
        let placed = row::place(&mut frags[i], h, st.page, true, false, mx)?;
        Some(placed.map(|r| r.placed).unwrap_or_default())
    } else {
        None
    };
    st.placed = vec![Placed {
        item,
        y: 0.0,
        height: h,
        first: frags[i].first,
        last: true,
        children,
    }];
    Ok(st.done(frags[i + 1..].to_vec()))
}

/// Splits a leaf so its first part takes `avail`: the two parts, or `None` to move it whole.
pub fn split_leaf(item: &ItemRef, avail: f64) -> Option<(ItemRef, ItemRef)> {
    let Item::Leaf(l) = &**item else { return None };
    let (cur, next) = match l.split {
        Split::No => return None,
        Split::Space { collapse } => {
            if !collapse && avail <= 0.0 {
                return None;
            }
            let fixed = |h| {
                Item::Leaf(super::items::Leaf {
                    height: h,
                    repeat: false,
                    data: None,
                    split: Split::No,
                })
            };
            let next = if collapse {
                fixed(0.0)
            } else {
                space(l.height - avail, false)
            };
            (fixed(avail), next)
        }
        Split::Node => {
            let node = &l.data.as_ref()?.node;
            let (a, b) = build::split(node, avail - node.layout.margin[crate::style::TOP])?;
            (flow::leaf(a), flow::leaf(b))
        }
    };
    Some((Rc::new(cur), Rc::new(next)))
}

/// Fresh copies of the repeating children that placed whole on this page, to head the part
/// that continues. Ones rendered from a lazy repeat the lazy, to render again.
pub fn repeats(fragment: &Fragment, inner: &FillResult, above: bool) -> Vec<Fragment> {
    let came_from_lazy = |item: &ItemRef| {
        fragment
            .children
            .iter()
            .any(|f| f.origin.is_some() && Rc::ptr_eq(&f.item, item))
    };
    // Only repeats placed, with nothing real above: repeating again would loop forever.
    if !above
        && !inner.placed.is_empty()
        && inner
            .placed
            .iter()
            .all(|p| p.item.is_repeat() || came_from_lazy(&p.item))
    {
        return Vec::new();
    }
    let mut sources: Vec<ItemRef> = Vec::new();
    for child in &fragment.children {
        if let Some(origin) = &child.origin {
            if sources.iter().any(|s| Rc::ptr_eq(s, origin)) {
                continue;
            }
            let all = fragment
                .children
                .iter()
                .filter(|f| f.origin.as_ref().is_some_and(|o| Rc::ptr_eq(o, origin)))
                .all(|f| fully_placed(&inner.placed, &f.item));
            if all {
                sources.push(origin.clone());
            }
            continue;
        }
        if child.item.is_repeat() && fully_placed(&inner.placed, &child.item) {
            sources.push(child.item.clone());
        }
    }
    to_fragments(&sources)
}
