//! Floats: each `float` goes to its side of the parent's content box, the text after it flows
//! around its box or `shapeOutside`, and `clear` moves a node below the floats before it. Both
//! change what Yoga measures, so a page with floats is laid out again until they settle.

use super::{Assets, is_container, text};
use crate::style::Dim;
use crate::style::shape::{Radius, ShapeOutside};
use crate::text::exclusion::{Exclusion, Shape};
use crate::tree::{Kind, Node};

/// Layouts after the first, at most, for the floats to settle.
const PASSES: usize = 3;

/// Whether anything in `n`, built or still waiting as JSON, floats.
pub fn has(n: &Node) -> bool {
    matches!(n.style.float.as_deref(), Some("left" | "right"))
        || n.pending.iter().any(|raw| raw.get().contains("\"float\""))
        || n.children.iter().any(has)
}

/// After a layout of `page`: while its floats change what text wraps around or how far `clear`
/// pushes, `again` lays it out once more. Pages without floats are left as they are.
pub fn settle(
    page: &mut Node,
    a: &Assets,
    mut again: impl FnMut(&mut Node) -> Result<(), String>,
) -> Result<(), String> {
    if !has(page) {
        return Ok(());
    }
    for pass in 0..=PASSES {
        if !resolve(page, a)? || pass == PASSES {
            break;
        }
        again(page)?;
    }
    Ok(())
}

/// Places the floats in `n`'s subtree and works out, from the boxes Yoga gave, what the text
/// after them flows around and each node's clearance. True when any of that changed.
fn resolve(n: &mut Node, a: &Assets) -> Result<bool, String> {
    if !is_container(n) {
        return Ok(false);
    }
    let l = n.layout;
    let (start, end) = (
        l.padding[3] + l.border[3],
        l.width - l.padding[1] - l.border[1],
    );
    let mut changed = false;
    let mut exclusions = Vec::new();
    // Each float's side and the bottom of its margin box, for `clear`.
    let mut edges: Vec<(bool, f64)> = Vec::new();
    for c in n
        .children
        .iter_mut()
        .filter(|c| c.kind != Kind::TextInstance)
    {
        let float = c.style.float.as_deref();
        if let Some(left) = float
            .filter(|f| matches!(*f, "left" | "right"))
            .map(|f| f == "left")
        {
            let cl = &mut c.layout;
            cl.left = if left {
                start + cl.margin[3]
            } else {
                end - cl.width - cl.margin[1]
            };
            exclusions.push(exclusion(c, left));
            edges.push((left, c.layout.top + c.layout.height + c.layout.margin[2]));
        } else {
            let clear = c.style.clear.as_deref().unwrap_or("none");
            let below = edges
                .iter()
                .filter(|(left, _)| {
                    clear == "both" || clear == if *left { "left" } else { "right" }
                })
                .map(|e| e.1)
                .fold(f64::NEG_INFINITY, f64::max);
            // Where the node would be without its clearance.
            let top = c.layout.top - c.clearance;
            let clearance = (below - top).max(0.0);
            if (clearance - c.clearance).abs() > 0.01 {
                c.clearance = clearance;
                changed = true;
            }
            if c.kind == Kind::Text {
                changed |= wrap(c, &exclusions, a)?;
            }
        }
        changed |= resolve(c, a)?;
    }
    Ok(changed)
}

/// Lays out a text's lines again, around `exclusions` (in its parent's coordinates) and with no
/// height limit, when they differ from those it flows around. A piece cut by a page break keeps
/// its lines.
fn wrap(t: &mut Node, exclusions: &[Exclusion], a: &Assets) -> Result<bool, String> {
    let Some(state) = t.text.clone() else {
        return Ok(false);
    };
    let l = t.layout;
    let mut inside = exclusions.to_vec();
    for e in &mut inside {
        e.shape
            .translate(-(l.left + l.padding[3]), -(l.top + l.padding[0]));
    }
    let Some(was) = state.borrow().wrapped.clone() else {
        return Ok(false);
    };
    if was == inside || state.borrow().lines.is_none() {
        return Ok(false);
    }
    let mut opts = text::options(t);
    opts.exclusions = inside;
    let width = l.width - l.padding[1] - l.padding[3];
    let lines = crate::text::layout(&text::paragraphs(t, a)?, width, None, &opts);
    let mut st = state.borrow_mut();
    st.lines = Some(lines);
    st.wrapped = Some(opts.exclusions);
    t.dirty = true;
    Ok(true)
}

/// What text keeps out of beside a float: its `shapeOutside`, else its box widened by the
/// margin that faces the text and lengthened by its bottom margin.
fn exclusion(f: &Node, left: bool) -> Exclusion {
    let l = f.layout;
    let [_, mr, mb, ml] = l.margin;
    let shape = match &f.style.shape_outside {
        Some(s) => {
            let mut s = shape(s, l.width, l.height);
            s.translate(l.left, l.top);
            s
        }
        None => Shape::Rect {
            x: l.left - if left { 0.0 } else { ml },
            y: l.top,
            w: l.width + if left { mr } else { ml },
            h: l.height + mb,
        },
    };
    Exclusion { shape, left }
}

fn scalar(d: Dim, of: f64) -> f64 {
    match d {
        Dim::Pt(x) => x,
        Dim::Pct(p) => p / 100.0 * of,
        Dim::Auto => 0.0,
    }
}

/// A shape in its float's box, `w` by `h` with the origin at its top left corner.
fn shape(s: &ShapeOutside, w: f64, h: f64) -> Shape {
    // A radius along one axis, from a center at `c` in a side `size` long.
    let axis = |r: Radius, c: f64, size: f64| match r {
        Radius::ClosestSide => c.min(size - c),
        Radius::FarthestSide => c.max(size - c),
        Radius::Len(d) => scalar(d, size),
    };
    match s {
        ShapeOutside::Circle { cx, cy, r } => {
            let (cx, cy) = (scalar(*cx, w), scalar(*cy, h));
            let sides = [cx, cy, w - cx, h - cy];
            let r = match *r {
                Radius::ClosestSide => sides.into_iter().fold(f64::INFINITY, f64::min),
                Radius::FarthestSide => sides.into_iter().fold(0.0, f64::max),
                // A percentage of a circle's radius is of the box's diagonal over √2.
                Radius::Len(d) => scalar(d, w.hypot(h) / std::f64::consts::SQRT_2),
            };
            Shape::Ellipse {
                cx,
                cy,
                rx: r,
                ry: r,
            }
        }
        ShapeOutside::Ellipse { cx, cy, rx, ry } => {
            let (cx, cy) = (scalar(*cx, w), scalar(*cy, h));
            Shape::Ellipse {
                cx,
                cy,
                rx: axis(*rx, cx, w),
                ry: axis(*ry, cy, h),
            }
        }
        ShapeOutside::Polygon(ps) => Shape::Polygon(
            ps.iter()
                .map(|(x, y)| (scalar(*x, w), scalar(*y, h)))
                .collect(),
        ),
        ShapeOutside::Inset([t, r, b, l]) => {
            let (x, y) = (scalar(*l, w), scalar(*t, h));
            Shape::Rect {
                x,
                y,
                w: w - x - scalar(*r, w),
                h: h - y - scalar(*b, h),
            }
        }
    }
}
