//! Pagination, the default algorithm of the components: each `<Page>` is laid out on an endless
//! sheet, then cut at the page height again and again. `fixed` nodes go to both sides of every
//! cut, `break`, `wrap={false}` and `minPresenceAhead` move nodes to the next page, and boxes and
//! text split between them.

mod bands;
pub mod page;
mod split;

use crate::RenderArgs;
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

impl<'a, 'b> Pager<'a, 'b> {
    /// Cuts one `<Page>` into pages, handing each to `done` as soon as it is cut. Returns how
    /// many pages it made. `first` is the document page number of the first one.
    pub fn paginate(
        &mut self,
        mut page: Node,
        first: u32,
        done: &mut Done<'a, 'b>,
    ) -> Result<u32, String> {
        if page.props.get("wrap") == Some(&serde_json::Value::Bool(false)) {
            layout_page(&mut page, None, self.assets)?;
            done(
                self,
                Finished {
                    page,
                    sub_index: 0,
                    last: true,
                },
            )?;
            return Ok(1);
        }
        let (height, _, pad_bottom) = page::heights(&page);
        let cut = height.unwrap_or(f64::NAN) - pad_bottom;
        let mut remainder = Remainder::new(&mut page, cut, self.assets)?;
        let mut count = 0;
        let mut rest = Some(page);
        while let Some(p) = rest {
            let (current, next) = self.split_page(&mut remainder, p, first + count)?;
            done(
                self,
                Finished {
                    page: current,
                    sub_index: count,
                    last: next.is_none(),
                },
            )?;
            count += 1;
            rest = next;
        }
        Ok(count)
    }

    fn split_page(
        &mut self,
        remainder: &mut Remainder,
        mut page: Node,
        number: u32,
    ) -> Result<(Node, Option<Node>), String> {
        let (height, _, pad_bottom) = page::heights(&page);
        let height = height.unwrap_or(f64::NAN);
        if page.dynamic {
            let args = RenderArgs {
                page_number: number,
                total_pages: None,
                sub_page_number: None,
                sub_page_total_pages: None,
                ..Default::default()
            };
            self.resolve_dynamic(&mut page, &args)?;
            remainder.relayout(&mut page, &[], height - pad_bottom, self.assets)?;
        }
        if let Some(base) = self.band_base
            && bands::fit(&mut page, base)
        {
            let (_, _, pad_bottom) = page::heights(&page);
            remainder.relayout(&mut page, &[], height - pad_bottom, self.assets)?;
        }
        let (_, pad_top, pad_bottom) = page::heights(&page);
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
        if next.iter().all(Node::is_fixed) && page.pending.is_empty() {
            return Ok((current, None));
        }
        page.props.shift_remove("bookmark");
        page.children = next;
        remainder.relayout(&mut page, &gone, height - pad_bottom, self.assets)?;
        Ok((current, Some(page)))
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
