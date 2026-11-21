//! Layout, computed by Yoga: a page's nodes become Yoga nodes, leaves are measured through
//! Yoga's callbacks, and the boxes are written back onto the nodes.

mod apply;
mod canvas;
pub mod image;
mod svg;
pub mod text;
pub mod yoga;

use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::FontStore;
use crate::style::Dim;
use crate::tree::node::{Layout, TextBox};
use crate::tree::{Kind, Node};
use yoga::{Edge, NodeId, Tree};

/// What leaves need to be measured.
pub struct Assets<'a> {
    pub fonts: &'a FontStore,
    /// Pixel size of an image asset, by the id the document gives it.
    pub image_size: &'a dyn Fn(u32) -> Option<(f64, f64)>,
    /// Runs a canvas's `paint` function at a size (NaN to only measure): painter calls as JSON.
    pub paint: &'a dyn Fn(u32, f64, f64) -> Option<String>,
    /// Runs a hyphenation callback on a word: its parts.
    pub hyphenate: &'a dyn Fn(u32, &str) -> Option<Vec<String>>,
    /// The document's own hyphenation callback, used by text without one.
    pub hyphenation: Option<u32>,
    /// Emoji images: the emoji text to its asset id.
    pub emoji: &'a [(String, u32)],
}

/// Lays out a page. `height` is the page box's height; `None` lays it out on an endless sheet.
pub fn layout_page(page: &mut Node, height: Option<f64>, a: &Assets) -> Result<(), String> {
    let mut tree = Tree::new();
    let mut ids = Vec::new();
    let page_area = height.map(|h| {
        let p = |i: usize| match page.style.padding[i] {
            Some(Dim::Pt(x)) => x,
            _ => 0.0,
        };
        h - p(0) - p(2)
    });
    let root = build(&mut tree, page, true, height, page_area, a, &mut ids)?;
    tree.calculate(root, None, None);
    let mut ids = ids.into_iter();
    write_back(&tree, page, &mut ids);
    drop(tree);
    fill_text(page, a)
}

/// Text that Yoga never measured (hidden, or sized by its style) gets lines for its box.
fn fill_text(n: &mut Node, a: &Assets) -> Result<(), String> {
    if n.kind == Kind::Text {
        let state = n.text.get_or_insert_with(Default::default).clone();
        if state.borrow().lines.is_none() {
            let l = &n.layout;
            let width = l.width - l.padding[1] - l.padding[3];
            let height = l.height - l.padding[0] - l.padding[2];
            let lines = crate::text::layout(
                &text::paragraphs(n, a)?,
                width,
                Some(height),
                &text::options(n),
            );
            state.borrow_mut().lines = Some(lines);
        }
        return Ok(());
    }
    if is_container(n) {
        for child in &mut n.children {
            fill_text(child, a)?;
        }
    }
    Ok(())
}

fn is_container(n: &Node) -> bool {
    !matches!(
        n.kind,
        Kind::Text | Kind::Note | Kind::Svg | Kind::TextInstance
    )
}

fn build(
    t: &mut Tree,
    n: &mut Node,
    is_page: bool,
    page_height: Option<f64>,
    page_area: Option<f64>,
    a: &Assets,
    ids: &mut Vec<NodeId>,
) -> Result<NodeId, String> {
    let id = t.node();
    ids.push(id);
    let height = if is_page {
        page_height.map(Dim::Pt)
    } else {
        n.style.height
    };
    apply::apply(t, id, &n.style, height);
    match n.kind {
        Kind::Text => {
            let state = n
                .text
                .get_or_insert_with(|| Rc::new(RefCell::new(TextBox::default())))
                .clone();
            let paragraphs = if state.borrow().lines.is_none() {
                Some(Rc::new(text::paragraphs(n, a)?))
            } else {
                None
            };
            let m = text::measure(
                state,
                paragraphs,
                text::options(n),
                n.style.text_align.clone(),
            );
            t.set_measure(id, Box::new(m));
        }
        Kind::Image => {
            if let Some(m) = image::measure(n, page_area, a) {
                t.set_measure(id, m);
            }
        }
        Kind::Svg => t.set_measure(id, svg::measure(n)),
        Kind::Canvas => {
            if let Some(m) = canvas::measure(n, page_area, a) {
                t.set_measure(id, m);
            }
        }
        _ => {}
    }
    if is_container(n) {
        for child in &mut n.children {
            if child.kind == Kind::TextInstance {
                continue;
            }
            let c = build(t, child, false, page_height, page_area, a, ids)?;
            t.push_child(id, c);
        }
    }
    Ok(id)
}

fn write_back(t: &Tree, n: &mut Node, ids: &mut impl Iterator<Item = NodeId>) {
    let id = ids.next().unwrap();
    let r = t.layout(id);
    let mut l = Layout {
        left: r.left as f64,
        top: r.top as f64,
        width: r.width as f64,
        height: r.height as f64,
        ..Layout::default()
    };
    for (i, edge) in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left]
        .into_iter()
        .enumerate()
    {
        let (m, p, b) = t.edges(id, edge);
        (l.margin[i], l.padding[i], l.border[i]) = (m as f64, p as f64, b as f64);
    }
    n.layout = l;
    if is_container(n) {
        for child in n
            .children
            .iter_mut()
            .filter(|c| c.kind != Kind::TextInstance)
        {
            write_back(t, child, ids);
        }
    }
}
