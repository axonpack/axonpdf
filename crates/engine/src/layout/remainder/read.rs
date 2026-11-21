//! Reading back the boxes the next cut looks at.

use super::has_more;
use crate::layout::yoga::Tree;
use crate::layout::{Assets, fill_text, is_container, write_back};
use crate::tree::{Kind, Node};

pub(super) enum Read {
    Whole,
    /// A window ends above the cut: it must grow.
    Short,
    /// A window is in it: what follows lies below the cut.
    Windowed,
}

/// Copies the boxes of `n`'s children, and of the children of those the cut crosses (their
/// tops are relative to `n`). Text that Yoga never measured gets its lines. Whatever follows a
/// window is put below the cut.
pub(super) fn read_near(t: &Tree, n: &mut Node, cut: f64, a: &Assets) -> Result<Read, String> {
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
