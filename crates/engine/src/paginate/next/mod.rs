//! The newer pagination engine, which a document opts into with `experimentalPagination` or a
//! `layout` template on any page. Each `<Page>`'s content is laid out once on an endless sheet
//! and kept with those boxes; then each page measures the room its template leaves, packs the
//! content into it as items (see [`fill`]), and is laid out again with what landed on it.
//! Pages are handed on one at a time.
//!
//! Where it differs from the components on purpose: `break` counts at any depth, not only on the
//! page's own children; a page with no height grows instead of failing for lack of room; and a
//! `render` node is measured under its own parents' styles, not the page's.
//!
//! ponytail: one `<Page>`'s whole content is laid out and held while it is cut (about 0.7 MB a
//! page of dense table), since its first boxes decide every break. A layout window like the
//! default engine's would keep it flat.

mod build;
mod cut;
mod fill;
mod flow;
mod items;
mod overflow;
mod row;
mod state;
mod template;

use serde_json::{Map, Value};

use super::{Callback, Pager, page};
use crate::RenderArgs;
use crate::layout::layout_page;
use crate::layout::remainder::has_more;
use crate::style::Container;
use crate::tree::{Node, RawNode, truthy};
use items::{Fragment, to_fragments};

/// What the newer engine reads off a page's JSON: its `layout` function, and whether it opts in.
#[derive(serde::Deserialize)]
pub struct Head {
    #[serde(default)]
    pub layout: Option<u32>,
    #[serde(default)]
    props: Map<String, Value>,
}

impl Head {
    pub fn read(json: &str) -> Head {
        serde_json::from_str(json).unwrap_or(Head {
            layout: None,
            props: Map::new(),
        })
    }

    /// Any page asking for it switches the whole document.
    pub fn wants_next(&self) -> bool {
        self.layout.is_some() || self.props.get("experimentalPagination").is_some_and(truthy)
    }
}

/// One `<Page>` being cut by the newer engine.
pub struct Cut {
    /// The page as built, until its content is taken out of it.
    built: Option<Node>,
    /// The page without its children, each page's starting point.
    shell: Node,
    layout: Option<u32>,
    /// Where the content sits in the first layout, and how many nodes it is.
    content: (Vec<usize>, usize),
    frags: Vec<Fragment>,
    /// The page's header and footer bands, and its own top and bottom padding they add to.
    bands: Vec<Node>,
    base: Option<(f64, f64)>,
    width: Option<f64>,
    numbers: Numbers,
    pub count: u32,
}

/// What a page's numbers are made from: the first page's number, the `<Page>`'s page count and
/// whether the total is filled in at the end.
#[derive(Clone, Copy)]
struct Numbers {
    first: u32,
    sub_total: Option<u32>,
    total_pending: bool,
}

impl Numbers {
    /// The numbers of the `n`th page cut from the `<Page>`.
    fn args(self, n: u32, total: Option<u32>) -> RenderArgs {
        RenderArgs {
            page_number: self.first + n - 1,
            total_pages: total,
            sub_page_number: Some(n),
            sub_page_total_pages: self.sub_total,
            is_last: None,
            total_pending: self.total_pending,
        }
    }
}

/// Builds a `<Page>` for the newer engine. A `layout` template, called for page 1, is laid
/// around its children, which inherit from where it puts them.
pub fn open(
    mut raw: RawNode,
    head: &Head,
    first: u32,
    (sub_total, total_pending): (Option<u32>, bool),
    call: &mut Callback,
) -> Result<(Cut, Container), String> {
    let mut content = (vec![0], usize::MAX);
    if let Some(id) = head.layout {
        let args = RenderArgs {
            page_number: 1,
            ..Default::default()
        };
        let made = call(id, &args)?;
        let children = std::mem::take(&mut raw.children);
        let count = children.len();
        let (mut found, mut path) = (0, Vec::new());
        raw.children = template::splice(made.children, &mut Some(children), &mut found, &mut path);
        template::check(found)?;
        content = (path, count);
    }
    let (mut node, c) = page::build(raw)?;
    let base = page::band_base(&node);
    let (bands, children) = std::mem::take(&mut node.children)
        .into_iter()
        .partition(|n| n.has(super::bands::BAND));
    let pending = std::mem::take(&mut node.pending);
    let shell = node.clone();
    (node.children, node.pending) = (children, pending);
    let cut = Cut {
        built: Some(node),
        shell,
        layout: head.layout,
        content,
        frags: Vec::new(),
        bands,
        base,
        width: None,
        numbers: Numbers {
            first,
            sub_total,
            total_pending,
        },
        count: 0,
    };
    Ok((cut, c))
}

impl Cut {
    /// Lays the page out once and takes its content out as items.
    fn start(&mut self, pager: &Pager) -> Result<(), String> {
        let Some(mut page) = self.built.take() else {
            return Ok(());
        };
        let fetched = fetch_all(&mut page, pager);
        if let Err(e) = fetched {
            self.built = Some(page);
            return Err(e);
        }
        layout_page(&mut page, None, pager.assets)?;
        let (path, count) = &self.content;
        let (at, down) = path.split_last().unwrap();
        let mut parent = &mut page;
        for &i in down {
            parent = &mut parent.children[i];
        }
        let end = (*at + count).min(parent.children.len());
        let content: Vec<Node> = parent.children.drain(*at..end).collect();
        self.frags = to_fragments(&[std::rc::Rc::new(flow::root(content))]);
        Ok(())
    }
}

/// Builds every child still waiting as JSON and fetches every row.
pub(super) fn fetch_all(n: &mut Node, pager: &Pager) -> Result<(), String> {
    while has_more(n) && (pager.assets.more_rows)(n)? {}
    n.children.iter_mut().try_for_each(|c| fetch_all(c, pager))
}
