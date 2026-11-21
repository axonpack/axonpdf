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

mod read;
mod sync;
mod window;

use read::{Read, read_near};
use sync::sync;
use window::Ctx;

use super::yoga::{Dim as YDim, FlexDirection, NodeId, Tree};
use super::{Assets, floats, full, is_container, write_back};
use crate::style::Dim;
use crate::tree::Node;

/// A `Rows` node's id for its batches, and the mark that it has sent them all.
pub const ROWS: &str = "rows";
pub const ROWS_DONE: &str = "__rows_done";

/// Whether more children are still to come: waiting as JSON, or from a `Rows` source.
pub fn has_more(n: &Node) -> bool {
    !n.pending.is_empty() || (n.has(ROWS) && !n.is(ROWS_DONE))
}

pub struct Remainder {
    tree: Tree,
    /// Whether the page has floats, once known: it is then laid out whole at every cut.
    floats: Option<bool>,
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

/// The Yoga nodes of a subtree, for freeing it.
pub fn ids(n: &Node, out: &mut Vec<NodeId>) {
    out.extend(n.yoga);
    if is_container(n) {
        n.children.iter().for_each(|c| ids(c, out));
    }
}
