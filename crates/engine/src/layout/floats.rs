//! Floats, after Yoga: each `float` goes to its side of the parent's content box, the text after
//! it flows around its box or `shapeOutside`, and `clear` moves a node below the floats before it.

use super::{Assets, is_container, text};
use crate::style::Dim;
use crate::style::shape::{Radius, ShapeOutside};
use crate::text::exclusion::{Exclusion, Shape};
use crate::tree::{Kind, Node};

/// Resolves the floats in `n`'s subtree.
pub fn resolve(n: &mut Node, a: &Assets) -> Result<(), String> {
    if !is_container(n) {
        return Ok(());
    }
    let l = n.layout;
    let (start, end) = (
        l.padding[3] + l.border[3],
        l.width - l.padding[1] - l.border[1],
    );
    let mut exclusions = Vec::new();
    // Each float's side and the bottom of its margin box, for `clear`.
    let mut edges: Vec<(bool, f64)> = Vec::new();
    let mut shift = 0.0;
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
            c.layout.top += shift;
            let clear = c.style.clear.as_deref().unwrap_or("none");
            let below = edges
                .iter()
                .filter(|(left, _)| {
                    clear == "both" || clear == if *left { "left" } else { "right" }
                })
                .map(|e| e.1)
                .fold(0.0, f64::max);
            let extra = (below - c.layout.top).max(0.0);
            c.layout.top += extra;
            shift += extra;
            if c.kind == Kind::Text {
                wrap(c, &exclusions, a)?;
            }
        }
        resolve(c, a)?;
    }
    // Cleared nodes moved down after Yoga ran: the parent grows to hold them.
    n.layout.height += shift;
    Ok(())
}

/// Lays out a text's lines around `exclusions` (in its parent's coordinates), with no height
/// limit; without any, it keeps the lines it has.
fn wrap(t: &mut Node, exclusions: &[Exclusion], a: &Assets) -> Result<(), String> {
    let Some(state) = t.text.clone() else {
        return Ok(());
    };
    if exclusions.is_empty() {
        state.borrow_mut().wrapped = None;
        return Ok(());
    }
    let l = t.layout;
    let mut opts = text::options(t);
    opts.exclusions = exclusions.to_vec();
    for e in &mut opts.exclusions {
        e.shape
            .translate(-(l.left + l.padding[3]), -(l.top + l.padding[0]));
    }
    let width = l.width - l.padding[1] - l.padding[3];
    let lines = crate::text::layout(&text::paragraphs(t, a)?, width, None, &opts);
    state.borrow_mut().wrapped = Some(lines);
    Ok(())
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
