//! How much of a long container is kept in Yoga: whether it may keep only a window of its
//! first children, and how many reach past the cut.

use crate::layout::yoga::{Align, FlexDirection, Justify, PositionType, Wrap};
use crate::style::Dim;
use crate::tree::Node;

/// Containers with fewer children are kept whole.
pub(super) const WINDOW_MIN: usize = 64;

/// What a container's children may rely on, from the page down.
#[derive(Clone, Copy)]
pub(super) struct Ctx {
    /// The width comes from above, not from the content.
    pub width_set: bool,
    /// No absolutely placed node (but fixed ones) on the way down.
    pub no_absolute: bool,
    /// Only telling Yoga again what it holds: windows stay as they are, no rows are fetched.
    pub again: bool,
}

/// Whether `n` may keep only a window of its children. A page's own height is not given to
/// Yoga while its rest is laid out.
pub(super) fn can_window(n: &Node, is_page: bool, ctx: Ctx) -> bool {
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
pub(super) fn in_flow(n: &Node) -> bool {
    n.children
        .iter()
        .all(|c| c.style.position != Some(PositionType::Absolute) || c.is_fixed())
}

/// What a child of `n` may rely on.
pub(super) fn child_ctx(n: &Node, c: &Node, ctx: Ctx) -> Ctx {
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
pub(super) fn window(n: &Node, reach: f64) -> usize {
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
