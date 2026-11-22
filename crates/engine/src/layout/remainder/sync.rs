//! Telling Yoga what a cut changed, and nothing else.

use super::has_more;
use super::window::{Ctx, WINDOW_MIN, can_window, child_ctx, in_flow, window};
use crate::layout::yoga::Tree;
use crate::layout::{Assets, apply, is_container, set_leaf_measure, set_text_measure};
use crate::tree::{Kind, Node};

/// Tells Yoga what changed: new nodes are built, changed ones get their style, lines and
/// children again, the windowed ones only their window. Unchanged subtrees are left alone.
pub(super) fn sync(
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
