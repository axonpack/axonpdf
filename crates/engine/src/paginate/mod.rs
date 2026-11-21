//! Pagination, the default algorithm of the components: each `<Page>` is laid out on an endless
//! sheet, then cut at the page height again and again. `fixed` nodes go to both sides of every
//! cut, `break`, `wrap={false}` and `minPresenceAhead` move nodes to the next page, and boxes and
//! text split between them.

mod bands;
pub mod next;
pub mod page;
mod split;

use crate::RenderArgs;
use crate::layout::yoga::NodeId;
use crate::layout::{Assets, Remainder, layout_page};
use crate::style::Container;
use crate::tree::{Kind, Node, RawNode, RenderResult, extras};

/// Calls a node's `render` function on the adapter's side.
pub type Callback<'a> = dyn FnMut(u32, &RenderArgs) -> Result<RenderResult, String> + 'a;

pub struct Pager<'a, 'b> {
    pub assets: &'a Assets<'a>,
    pub callback: &'a mut Callback<'b>,
    pub container: Container,
    /// The document's page count when a first run counted it.
    pub total: Option<u32>,
    /// The page's own top and bottom padding, when it has bands to add to them.
    pub band_base: Option<(f64, f64)>,
}

/// One finished page, laid out at its final size.
pub struct Finished {
    pub page: Node,
    /// 0-based index among the pages cut from the same `<Page>`.
    pub sub_index: u32,
    /// Nothing of its `<Page>` is left for another page.
    pub last: bool,
}

/// Takes each page as it is cut, with the pager to finish it.
pub type Done<'a, 'b> = dyn FnMut(&mut Pager<'a, 'b>, Finished) -> Result<(), String> + 'a;

/// One `<Page>` being cut, a page at a time. It can wait between pages, and when a page needs
/// rows that have not come yet, the same call is made again once they have.
pub struct Cut {
    /// What is left to cut.
    page: Option<Node>,
    remainder: Remainder,
    first: u32,
    count: u32,
    /// The rest must be laid out again first; the Yoga nodes of what left with the last page.
    relayout: Option<Vec<NodeId>>,
    /// `wrap={false}`: the page as it is.
    whole: bool,
    /// The document uses the newer engine.
    next: Option<next::Cut>,
}

/// A `<Page>` ready to cut: the cut, the page box, and the page's own padding when it has bands.
pub type Opened = (Cut, Container, Option<(f64, f64)>);

/// Builds a `<Page>` from its JSON and starts cutting it. `next` cuts it with the newer engine; `sub_total` is its page
/// count from a first run, `total_pending` whether the total is filled in at the end.
pub fn open(
    raw: RawNode,
    json: &str,
    next: bool,
    first: u32,
    totals: (Option<u32>, bool),
    call: &mut Callback,
) -> Result<Opened, String> {
    if next {
        let (cut, c) = next::open(raw, &next::Head::read(json), first, totals, call)?;
        let mut legacy = begin(None, first);
        legacy.next = Some(cut);
        return Ok((legacy, c, None));
    }
    let (node, c) = page::build(raw)?;
    let base = page::band_base(&node);
    Ok((begin(Some(node), first), c, base))
}

/// Starts cutting `page`, whose first page has number `first`.
fn begin(page: Option<Node>, first: u32) -> Cut {
    let whole = page
        .as_ref()
        .is_some_and(|p| p.props.get("wrap") == Some(&serde_json::Value::Bool(false)));
    Cut {
        page,
        remainder: Remainder::empty(),
        first,
        count: 0,
        relayout: Some(Vec::new()),
        whole,
        next: None,
    }
}

impl<'a, 'b> Pager<'a, 'b> {
    /// The next page of `cut`, or `None` when nothing is left. After an error the call can be
    /// made again (it is how a wait for rows comes out).
    pub fn next(&mut self, cut: &mut Cut) -> Result<Option<Finished>, String> {
        if let Some(n) = &mut cut.next {
            let f = n.next(self)?;
            cut.count = n.count;
            return Ok(f);
        }
        let Some(mut page) = cut.page.take() else {
            return Ok(None);
        };
        if cut.whole {
            layout_page(&mut page, None, self.assets)?;
            cut.count = 1;
            return Ok(Some(Finished {
                page,
                sub_index: 0,
                last: true,
            }));
        }
        let number = cut.first + cut.count;
        let step = (|| {
            if let Some(gone) = &mut cut.relayout {
                let (height, _, pad_bottom) = page::heights(&page);
                let at = height.unwrap_or(f64::NAN) - pad_bottom;
                cut.remainder.relayout(&mut page, gone, at, self.assets)?;
            }
            cut.relayout = None;
            self.split_page(&mut cut.remainder, &mut page, number)
        })();
        match step {
            Err(e) => {
                cut.page = Some(page);
                Err(e)
            }
            Ok((current, gone, more)) => {
                cut.count += 1;
                if more {
                    cut.relayout = Some(gone);
                    cut.page = Some(page);
                }
                Ok(Some(Finished {
                    page: current,
                    sub_index: cut.count - 1,
                    last: !more,
                }))
            }
        }
    }

    /// How many pages `cut` made.
    pub fn count(cut: &Cut) -> u32 {
        cut.count
    }

    /// Cuts one `<Page>` into pages, handing each to `done` as soon as it is cut. Returns how
    /// many pages it made.
    pub fn paginate(&mut self, mut cut: Cut, done: &mut Done<'a, 'b>) -> Result<u32, String> {
        while let Some(f) = self.next(&mut cut)? {
            done(self, f)?;
        }
        Ok(cut.count)
    }

    /// Cuts the next page off `page`, which keeps the rest. Returns the page, the Yoga nodes of
    /// what left with it, and whether anything is left.
    fn split_page(
        &mut self,
        remainder: &mut Remainder,
        page: &mut Node,
        number: u32,
    ) -> Result<(Node, Vec<NodeId>, bool), String> {
        let (height, _, pad_bottom) = page::heights(page);
        let height = height.unwrap_or(f64::NAN);
        if page.dynamic {
            let args = RenderArgs {
                page_number: number,
                ..Default::default()
            };
            self.resolve_dynamic(page, &args)?;
            remainder.relayout(page, &mut Vec::new(), height - pad_bottom, self.assets)?;
        }
        if let Some(base) = self.band_base
            && bands::fit(page, base)
        {
            let (_, _, pad_bottom) = page::heights(page);
            remainder.relayout(page, &mut Vec::new(), height - pad_bottom, self.assets)?;
        }
        let (_, pad_top, pad_bottom) = page::heights(page);
        let children = std::mem::take(&mut page.children);
        let mut gone = Vec::new();
        let (cur, next) = split::split_nodes(
            height - pad_bottom,
            height - pad_top - pad_bottom,
            children,
            &mut gone,
        );

        // What still waits as JSON stays with the rest of the page.
        let pending = std::mem::take(&mut page.pending);
        let mut current = page.clone();
        page.pending = pending;
        current.children = cur;
        build_all(&mut current, &self.container);
        layout_page(
            &mut current,
            Some(height).filter(|h| !h.is_nan()),
            self.assets,
        )?;
        let more = !(next.iter().all(Node::is_fixed) && page.pending.is_empty());
        page.props.shift_remove("bookmark");
        page.children = next;
        Ok((current, gone, more))
    }

    /// Runs `render` functions once more with the final numbers, and lays the page out again.
    /// The cut stays where it was even if the content grew.
    pub fn finish(&mut self, page: &mut Node, args: &RenderArgs) -> Result<(), String> {
        // A total filled in at the end gets room for one digit more than this page number.
        self.assets
            .total_digits
            .set(args.page_number.to_string().len() + 1);
        if page.dynamic {
            self.resolve_dynamic(page, args)?;
            let (height, ..) = page::heights(page);
            let height =
                height.filter(|_| page.props.get("wrap") != Some(&serde_json::Value::Bool(false)));
            layout_page(page, height, self.assets)?;
        }
        Ok(())
    }

    /// Calls the `render` functions under `node`, replacing their children, and marks the way
    /// down to them as changed.
    fn resolve_dynamic(&mut self, node: &mut Node, args: &RenderArgs) -> Result<(), String> {
        if !node.dynamic {
            return Ok(());
        }
        node.dirty = true;
        if let Some(id) = node.render {
            let result = (self.callback)(id, args)?;
            node.set_children(result.children, &self.container);
        }
        if let Some(what) = node.props.get(extras::COUNTER).and_then(|v| v.as_str()) {
            // While pages are being counted the total is not known; a digit holds its place.
            // Without a count the writer fills it in at the end.
            let n = if what == "page" {
                Some(args.page_number)
            } else {
                self.total
            };
            let value = match n {
                Some(n) => n.to_string(),
                None if args.total_pending => crate::TOTAL_PAGES.to_string(),
                None => "0".to_string(),
            };
            node.children = vec![Node::build(
                RawNode::text(value),
                &self.container,
                Some((&node.style, true)),
            )];
        }
        // Text holding something that changed lays out its lines again.
        if node.kind == Kind::Text {
            node.text = None;
        }
        for child in &mut node.children {
            self.resolve_dynamic(child, args)?;
        }
        Ok(())
    }
}

/// Builds whatever still waits as JSON in a finished page; the cut builds what it reaches, so
/// this normally finds nothing.
fn build_all(n: &mut Node, c: &Container) {
    if !n.pending.is_empty() {
        n.build_pending(c, usize::MAX);
        n.dirty = true;
    }
    n.children.iter_mut().for_each(|ch| build_all(ch, c));
}
