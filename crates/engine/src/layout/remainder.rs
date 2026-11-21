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

use super::yoga::{Align, FlexDirection, Justify, NodeId, PositionType, Tree, Wrap};
use super::{
    Assets, apply, build, fill_text, is_container, set_leaf_measure, set_text_measure, write_back,
};
use crate::style::Dim;
use crate::tree::{Kind, Node};

/// Containers with fewer children are kept whole.
const WINDOW_MIN: usize = 64;

pub struct Remainder {
    tree: Tree,
}

/// What a container's children may rely on, from the page down.
#[derive(Clone, Copy)]
struct Ctx {
    /// The width comes from above, not from the content.
    width_set: bool,
    /// No absolutely placed node (but fixed ones) on the way down.
    no_absolute: bool,
}

impl Remainder {
    /// Lays out a `<Page>` on an endless sheet, as far as the first cut at `cut` needs.
    pub fn new(page: &mut Node, cut: f64, a: &Assets) -> Result<Self, String> {
        let mut r = Remainder { tree: Tree::new() };
        r.relayout(page, &[], cut, a)?;
        Ok(r)
    }

    /// Everything laid out, as the components do it: the check the kept layout is held to.
    fn whole(page: &mut Node, a: &Assets) -> Result<Self, String> {
        let mut tree = Tree::new();
        let root = build(&mut tree, page, true, None, None, a)?;
        tree.calculate(root, None, None);
        write_back(&tree, page, true);
        fill_text(page, a)?;
        Ok(Remainder { tree })
    }

    /// Lays out what is left after a cut, `gone` being the Yoga nodes of what left with the
    /// finished page, and reads the boxes a cut at `cut` will look at.
    pub fn relayout(
        &mut self,
        page: &mut Node,
        gone: &[NodeId],
        cut: f64,
        a: &Assets,
    ) -> Result<(), String> {
        // A check switch: lay the rest out from scratch, as the components do, to compare.
        if std::env::var_os("AXONPDF_FRESH_LAYOUT").is_some() {
            *self = Remainder::whole(page, a)?;
            return Ok(());
        }
        let ctx = Ctx {
            width_set: true,
            no_absolute: true,
        };
        let mut slack = 2.0 * cut.max(200.0);
        let mut first = true;
        loop {
            page.dirty = true;
            sync(&mut self.tree, page, true, cut, slack, ctx, a)?;
            if first {
                gone.iter().for_each(|&id| self.tree.free(id));
                first = false;
            }
            self.tree.calculate(page.yoga.unwrap(), None, None);
            write_back(&self.tree, page, false);
            match read_near(&self.tree, page, cut, a)? {
                Read::Short => slack *= 4.0,
                _ => return Ok(()),
            }
        }
    }
}

/// Whether `n` may keep only a window of its children. A page's own height is not given to
/// Yoga while its rest is laid out.
fn can_window(n: &Node, is_page: bool, ctx: Ctx) -> bool {
    let s = &n.style;
    ctx.width_set
        && ctx.no_absolute
        && n.children.len() > WINDOW_MIN
        && matches!(s.flex_direction, None | Some(FlexDirection::Column))
        && matches!(s.flex_wrap, None | Some(Wrap::NoWrap))
        && matches!(s.justify_content, None | Some(Justify::FlexStart))
        && (is_page || s.height.is_none())
        && s.min_height.is_none()
        && s.max_height.is_none()
}

/// Whether none of `n`'s children is absolutely placed (fixed ones aside, with `fixed_ok`).
fn in_flow(n: &Node, fixed_ok: bool) -> bool {
    n.children
        .iter()
        .all(|c| c.style.position != Some(PositionType::Absolute) || (fixed_ok && c.is_fixed()))
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
    }
}

/// How many first children reach `reach` below the top, by their last known boxes; one
/// never laid out counts as the average of those that were (20pt if none was).
fn window(n: &Node, reach: f64) -> usize {
    let (mut known, mut sum) = (0.0, 0.0);
    let mut bottom = 0.0;
    for (i, c) in n.children.iter().enumerate() {
        if bottom > reach {
            return i;
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
    apply::apply(t, id, &n.style, if is_page { None } else { n.style.height });
    if is_container(n) {
        let ctx = Ctx {
            no_absolute: ctx.no_absolute && in_flow(n, true),
            ..ctx
        };
        n.window = if can_window(n, is_page, ctx) && in_flow(n, false) {
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
    if shown < n.children.len() {
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
