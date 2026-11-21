//! The rest of a long `<Page>`, kept in one Yoga tree from cut to cut.
//!
//! After each cut the components lay out everything left over again, so a long page costs
//! pages × nodes. The boxes must stay the same, so this keeps Yoga's own tree: only what the
//! cut changed (the split boxes along the cut, their new children lists, cut text) is told to
//! Yoga, whose cache gives every untouched subtree the layout it had. Nodes that left for the
//! finished page are freed, and only the boxes the next cut reads are copied back.
//!
//! A container with many children whose boxes cannot depend on the ones below them (a column
//! that does not wrap, packed at the start, with no height of its own, a width that does not
//! come from its content, and nothing absolute around it) keeps only a window of its first
//! children in Yoga, enough to pass the next cut. What lies past the window is below the cut.

use super::yoga::{Align, Dim as YDim, FlexDirection, Justify, NodeId, PositionType, Tree, Wrap};
use super::{
    Assets, apply, fill_text, floats, full, is_container, set_leaf_measure, set_text_measure,
    write_back,
};
use crate::style::Dim;
use crate::tree::{Kind, Node};

/// A `Rows` node's id for its batches, and the mark that it has sent them all.
pub const ROWS: &str = "rows";
pub const ROWS_DONE: &str = "__rows_done";

/// Whether more children are still to come: waiting as JSON, or from a `Rows` source.
pub fn has_more(n: &Node) -> bool {
    !n.pending.is_empty() || (n.has(ROWS) && !n.is(ROWS_DONE))
}

/// Containers with fewer children are kept whole.
const WINDOW_MIN: usize = 64;

pub struct Remainder {
    tree: Tree,
    /// Whether the page has floats, once known: it is then laid out whole at every cut.
    floats: Option<bool>,
}

/// What a container's children may rely on, from the page down.
#[derive(Clone, Copy)]
struct Ctx {
    /// The width comes from above, not from the content.
    width_set: bool,
    /// No absolutely placed node (but fixed ones) on the way down.
    no_absolute: bool,
    /// Only telling Yoga again what it holds: windows stay as they are, no rows are fetched.
    again: bool,
}

impl Remainder {
    /// Nothing laid out yet: the first [`Remainder::relayout`] lays out the page on an endless
    /// sheet, as far as its first cut needs.
    pub fn empty() -> Self {
        Remainder {
            tree: Tree::new(),
            floats: None,
        }
    }

    /// Everything laid out, as the components do it: the check the kept layout is held to, and
    /// the layout of a page with floats, which move what is below them.
    fn whole(page: &mut Node, sized: bool, a: &Assets) -> Result<Tree, String> {
        fn fetch_all(n: &mut Node, a: &Assets) -> Result<(), String> {
            while has_more(n) && (a.more_rows)(n)? {}
            n.children.iter_mut().try_for_each(|c| fetch_all(c, a))
        }
        fetch_all(page, a)?;
        let pass = |p: &mut Node| {
            let tree = full(p, None, a)?;
            if !sized {
                return Ok(tree);
            }
            let height = p.layout.height;
            full(p, Some(height), a)
        };
        let mut tree = pass(page)?;
        floats::settle(page, a, |p| {
            tree = pass(p)?;
            Ok(())
        })?;
        Ok(tree)
    }

    /// Lays out what is left after a cut, `gone` being the Yoga nodes of what left with the
    /// finished page, and reads the boxes a cut at `cut` will look at.
    /// After an error it can be called again; `gone` is emptied once freed.
    /// `sized` lays the page out once more at the height it came to, as the components do
    /// before cutting a page with dynamic content: Yoga sizes some boxes differently (`flex: 1`
    /// rows in stretched columns) once the page has a height.
    pub fn relayout(
        &mut self,
        page: &mut Node,
        gone: &mut Vec<NodeId>,
        cut: f64,
        sized: bool,
        a: &Assets,
    ) -> Result<(), String> {
        // A check switch for native debug builds (a WebAssembly build sees no environment): lay
        // the rest out from scratch, as the components do, to compare.
        let fresh = cfg!(debug_assertions) && std::env::var_os("AXONPDF_FRESH_LAYOUT").is_some();
        if fresh || *self.floats.get_or_insert_with(|| floats::has(page)) {
            self.tree = Remainder::whole(page, sized, a)?;
            return Ok(());
        }
        // Rows that come in batches may bring the first float.
        let found = std::cell::Cell::new(false);
        let more = |n: &mut Node| {
            let from = n.children.len();
            let more = (a.more_rows)(n)?;
            let new = n.children.get(from..).unwrap_or_default();
            found.set(found.get() || new.iter().any(floats::has));
            Ok(more)
        };
        let a = &Assets {
            more_rows: &more,
            ..*a
        };
        let ctx = Ctx {
            width_set: true,
            no_absolute: true,
            again: false,
        };
        let mut slack = 0.5 * cut.max(200.0);
        loop {
            page.dirty = true;
            let synced = sync(&mut self.tree, page, true, cut, slack, ctx, a);
            if found.get() {
                // Kept even when rows are still awaited, for the call made once they come.
                self.floats = Some(true);
            }
            synced?;
            gone.drain(..).for_each(|id| self.tree.free(id));
            let root = page.yoga.unwrap();
            self.tree.height(root, YDim::Auto);
            self.tree.calculate(root, None, None);
            // Again at the height it came to, as the components lay it out, where that can change
            // a box.
            let height = self.tree.layout(root).height;
            if sized && touch(page, false) {
                let again = Ctx { again: true, ..ctx };
                sync(&mut self.tree, page, true, cut, slack, again, a)?;
                self.tree.height(root, YDim::Pt(height));
                self.tree.calculate(root, None, None);
            }
            write_back(&self.tree, page, false);
            match read_near(&self.tree, page, cut, a)? {
                Read::Short => slack *= 4.0,
                _ => break,
            }
        }
        if found.get() {
            self.tree = Remainder::whole(page, sized, a)?;
        }
        Ok(())
    }
}

/// Marks the nodes Yoga holds whose boxes may change once the page has a height, and the way
/// down to them, as changed; returns whether there are any. Those are a set flex basis down a
/// column (Yoga keeps the basis it first worked out, and works it out differently with a
/// height) and percent heights. The next sync gives them their style and children again.
/// ponytail: by style, not proof; add a case here when a page with dynamic content still differs.
fn touch(n: &mut Node, in_column: bool) -> bool {
    if n.yoga.is_none() {
        return false;
    }
    let s = &n.style;
    let pct = |d: Option<Dim>| matches!(d, Some(Dim::Pct(_)));
    let mut changed = (in_column && matches!(s.flex_basis, Some(Dim::Pt(_) | Dim::Pct(_))))
        || pct(s.height)
        || pct(s.min_height)
        || pct(s.max_height);
    let column = matches!(
        s.flex_direction,
        None | Some(FlexDirection::Column | FlexDirection::ColumnReverse)
    );
    for c in &mut n.children {
        changed |= touch(c, column);
    }
    n.dirty |= changed;
    changed
}

/// Whether `n` may keep only a window of its children. A page's own height is not given to
/// Yoga while its rest is laid out.
fn can_window(n: &Node, is_page: bool, ctx: Ctx) -> bool {
    let s = &n.style;
    ctx.width_set
        && ctx.no_absolute
        && matches!(s.flex_direction, None | Some(FlexDirection::Column))
        && matches!(s.flex_wrap, None | Some(Wrap::NoWrap))
        && matches!(s.justify_content, None | Some(Justify::FlexStart))
        && (is_page || s.height.is_none())
        && s.min_height.is_none()
        && s.max_height.is_none()
}

/// Whether none of `n`'s children is absolutely placed but fixed ones, which a cut copies to
/// both sides whatever their place.
fn in_flow(n: &Node) -> bool {
    n.children
        .iter()
        .all(|c| c.style.position != Some(PositionType::Absolute) || c.is_fixed())
}

/// What a child of `n` may rely on.
fn child_ctx(n: &Node, c: &Node, ctx: Ctx) -> Ctx {
    let column = matches!(n.style.flex_direction, None | Some(FlexDirection::Column));
    let stretched = matches!(n.style.align_items, None | Some(Align::Stretch))
        && matches!(
            c.style.align_self,
            None | Some(Align::Auto | Align::Stretch)
        );
    let fixed_width = matches!(c.style.width, Some(Dim::Pt(_)));
    Ctx {
        width_set: fixed_width
            || (ctx.width_set
                && column
                && stretched
                && c.style.width.is_none()
                && c.style.aspect_ratio.is_none()),
        no_absolute: ctx.no_absolute,
        again: ctx.again,
    }
}

/// How many first children reach `reach` below the top, by their last known boxes; one
/// never laid out counts as the average of those that were (20pt if none was).
fn window(n: &Node, reach: f64) -> usize {
    let (mut known, mut sum) = (0.0, 0.0);
    let mut bottom = 0.0;
    for (i, c) in n.children.iter().enumerate() {
        if bottom > reach {
            // One at least: none would mean no window.
            return i.max(1);
        }
        let l = &c.layout;
        bottom += if c.yoga.is_some() {
            let h = l.margin[0] + l.height + l.margin[2];
            (known, sum) = (known + 1.0, sum + h);
            h
        } else if known > 0.0 {
            sum / known
        } else {
            20.0
        };
    }
    n.children.len()
}

/// Tells Yoga what changed: new nodes are built, changed ones get their style, lines and
/// children again, the windowed ones only their window. Unchanged subtrees are left alone.
fn sync(
    t: &mut Tree,
    n: &mut Node,
    is_page: bool,
    cut: f64,
    slack: f64,
    ctx: Ctx,
    a: &Assets,
) -> Result<(), String> {
    // A node left half done (waiting for rows) is done again on the next call.
    let r = sync_node(t, n, is_page, cut, slack, ctx, a);
    if r.is_err() {
        n.dirty = true;
    }
    r
}

fn sync_node(
    t: &mut Tree,
    n: &mut Node,
    is_page: bool,
    cut: f64,
    slack: f64,
    ctx: Ctx,
    a: &Assets,
) -> Result<(), String> {
    let id = match n.yoga {
        Some(_) if !n.dirty => return Ok(()),
        Some(id) => {
            if n.kind == Kind::Text {
                set_text_measure(t, id, n, a)?;
                t.mark_dirty(id);
            }
            id
        }
        None => {
            let id = t.node();
            n.yoga = Some(id);
            set_leaf_measure(t, id, n, None, a)?;
            id
        }
    };
    n.dirty = false;
    apply::apply(
        t,
        id,
        &n.style,
        if is_page { None } else { n.style.height },
        n.clearance,
    );
    if is_container(n) {
        let ctx = Ctx {
            no_absolute: ctx.no_absolute && in_flow(n),
            ..ctx
        };
        // Rows that arrive in batches are fetched until the window is full, or all of them
        // when there is no window.
        if has_more(n) && !ctx.again {
            let windowed = can_window(n, is_page, ctx);
            while !windowed || window(n, cut + slack) >= n.children.len() {
                if !(a.more_rows)(n)? {
                    break;
                }
            }
        }
        // Rows come in batches, so they window however few have come yet.
        let long = n.children.len() > WINDOW_MIN || has_more(n);
        n.window = if ctx.again {
            n.window
        } else if long && can_window(n, is_page, ctx) {
            window(n, cut + slack)
        } else {
            0
        };
        let shown = if n.window == 0 {
            n.children.len()
        } else {
            n.window
        };
        t.clear_children(id);
        for i in 0..shown {
            let c_ctx = child_ctx(n, &n.children[i], ctx);
            let child = &mut n.children[i];
            if child.kind == Kind::TextInstance {
                continue;
            }
            sync(t, child, false, cut - child.layout.top, slack, c_ctx, a)?;
            t.push_child(id, child.yoga.unwrap());
        }
    }
    Ok(())
}

enum Read {
    Whole,
    /// A window ends above the cut: it must grow.
    Short,
    /// A window is in it: what follows lies below the cut.
    Windowed,
}

/// Copies the boxes of `n`'s children, and of the children of those the cut crosses (their
/// tops are relative to `n`). Text that Yoga never measured gets its lines. Whatever follows a
/// window is put below the cut.
fn read_near(t: &Tree, n: &mut Node, cut: f64, a: &Assets) -> Result<Read, String> {
    if !is_container(n) {
        return Ok(Read::Whole);
    }
    let shown = if n.window == 0 {
        n.children.len()
    } else {
        n.window
    };
    let mut result = Read::Whole;
    let mut bottom = f64::NEG_INFINITY;
    for (i, child) in n.children.iter_mut().enumerate() {
        if child.kind == Kind::TextInstance {
            continue;
        }
        if i >= shown || matches!(result, Read::Windowed) {
            child.layout.top = f64::INFINITY;
            continue;
        }
        write_back(t, child, false);
        let l = child.layout;
        bottom = bottom.max(l.top + l.height + l.margin[2]);
        if l.top <= cut && l.top + l.height >= cut {
            if child.kind == Kind::Text {
                fill_text(child, a)?;
            }
            match read_near(t, child, cut - l.top, a)? {
                Read::Short => {
                    child.dirty = true;
                    return Ok(Read::Short);
                }
                Read::Windowed => result = Read::Windowed,
                Read::Whole => {}
            }
        }
    }
    if shown < n.children.len() || has_more(n) {
        if bottom <= cut + 1.0 {
            return Ok(Read::Short);
        }
        result = Read::Windowed;
    }
    Ok(result)
}

/// The Yoga nodes of a subtree, for freeing it.
pub fn ids(n: &Node, out: &mut Vec<NodeId>) {
    out.extend(n.yoga);
    if is_container(n) {
        n.children.iter().for_each(|c| ids(c, out));
    }
}
