//! A resolved style onto a Yoga node, with the components' defaults and quirks.

use super::yoga::{self, Align, Display, Edge, FlexDirection, Gutter, Len, NodeId, Tree, Wrap};
use crate::style::{Dim, Style};

fn dim(d: Dim) -> yoga::Dim {
    match d {
        Dim::Auto => yoga::Dim::Auto,
        Dim::Pt(x) => yoga::Dim::Pt(x as f32),
        Dim::Pct(x) => yoga::Dim::Pct(x as f32),
    }
}

/// Min, max, padding and gap take no `auto`; an `auto` there sets nothing.
fn len(d: Dim) -> Option<Len> {
    match d {
        Dim::Auto => None,
        Dim::Pt(x) => Some(Len::Pt(x as f32)),
        Dim::Pct(x) => Some(Len::Pct(x as f32)),
    }
}

const EDGES: [Edge; 4] = [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];

/// `height` is the page's own box height for a page, else the style's. `clearance` is added to
/// a top margin in points (`clear`).
pub fn apply(t: &mut Tree, id: NodeId, s: &Style, height: Option<Dim>, clearance: f64) {
    if let Some(h) = height {
        t.height(id, dim(h));
    }
    if let Some(w) = s.width {
        t.width(id, dim(w));
    }
    for (v, f) in [
        (s.min_width, Tree::min_width as fn(&mut Tree, NodeId, Len)),
        (s.max_width, Tree::max_width),
        (s.min_height, Tree::min_height),
        (s.max_height, Tree::max_height),
    ] {
        if let Some(l) = v.and_then(len) {
            f(t, id, l);
        }
    }
    for (i, edge) in EDGES.into_iter().enumerate() {
        let m = match s.margin[i] {
            Some(Dim::Pt(x)) if i == 0 => Some(Dim::Pt(x + clearance)),
            None if i == 0 && clearance > 0.0 => Some(Dim::Pt(clearance)),
            m => m,
        };
        if let Some(m) = m {
            t.margin(id, edge, dim(m));
        }
        if let Some(p) = s.padding[i].and_then(len) {
            t.padding(id, edge, p);
        }
        if let Some(b) = s.border_width[i] {
            t.border(id, edge, b as f32);
        }
        if let Some(p) = s.inset[i] {
            t.position(id, edge, dim(p));
        }
    }
    if let Some(p) = s.position {
        t.position_type(id, p);
    }
    t.display(
        id,
        if s.display_none {
            Display::None
        } else {
            Display::Flex
        },
    );
    t.flex_direction(id, s.flex_direction.unwrap_or(FlexDirection::Column));
    t.align_self(id, s.align_self.unwrap_or(Align::Auto));
    t.align_content(id, s.align_content.unwrap_or(Align::Auto));
    t.align_items(id, s.align_items.unwrap_or(Align::Stretch));
    if let Some(j) = s.justify_content {
        t.justify_content(id, j);
    }
    t.flex_wrap(id, s.flex_wrap.unwrap_or(Wrap::NoWrap));
    if let Some(o) = s.overflow {
        t.overflow(id, o);
    }
    if let Some(r) = s.aspect_ratio {
        t.aspect_ratio(id, r as f32);
    }
    if let Some(b) = s.flex_basis {
        t.flex_basis(id, dim(b));
    }
    // `flexShrink: 0` (or unset) shrinks anyway: zero reads as "not set" and becomes 1.
    let or = |v: Option<f64>, d: f64| v.filter(|x| *x != 0.0 && !x.is_nan()).unwrap_or(d);
    t.flex_grow(id, or(s.flex_grow, 0.0) as f32);
    t.flex_shrink(id, or(s.flex_shrink, 1.0) as f32);
    if let Some(g) = s.row_gap.and_then(len) {
        t.gap(id, Gutter::Row, g);
    }
    if let Some(g) = s.column_gap.and_then(len) {
        t.gap(id, Gutter::Column, g);
    }
}
