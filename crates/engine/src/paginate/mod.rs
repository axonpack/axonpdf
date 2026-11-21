//! Pagination, the default algorithm of the components: each `<Page>` is laid out on an endless
//! sheet, then cut at the page height again and again. `fixed` nodes go to both sides of every
//! cut, `break`, `wrap={false}` and `minPresenceAhead` move nodes to the next page, and boxes and
//! text split between them.

pub mod page;
mod split;

use crate::RenderArgs;
use crate::layout::{Assets, layout_page};
use crate::style::Container;
use crate::tree::{Node, RenderResult};

/// Calls a node's `render` function on the adapter's side.
pub type Callback<'a> = dyn FnMut(u32, &RenderArgs) -> Result<RenderResult, String> + 'a;

pub struct Pager<'a, 'b> {
    pub assets: &'a Assets<'a>,
    pub callback: &'a mut Callback<'b>,
    pub container: Container,
}

/// One finished page, laid out at its final size.
pub struct Finished {
    pub page: Node,
    /// 0-based index among the pages cut from the same `<Page>`.
    pub sub_index: u32,
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
        layout_page(&mut page, None, self.assets)?;
        if page.props.get("wrap") == Some(&serde_json::Value::Bool(false)) {
            done(self, Finished { page, sub_index: 0 })?;
            return Ok(1);
        }
        let mut count = 0;
        let mut rest = Some(page);
        while let Some(p) = rest {
            let (current, next) = self.split_page(p, first + count)?;
            done(
                self,
                Finished {
                    page: current,
                    sub_index: count,
                },
            )?;
            count += 1;
            rest = next;
        }
        Ok(count)
    }

    fn split_page(&mut self, mut page: Node, number: u32) -> Result<(Node, Option<Node>), String> {
        let (height, pad_top, pad_bottom) = page::heights(&page);
        let height = height.unwrap_or(f64::NAN);
        if has_dynamic(&page) {
            let args = RenderArgs {
                page_number: number,
                total_pages: None,
                sub_page_number: None,
                sub_page_total_pages: None,
            };
            self.resolve_dynamic(&mut page, &args)?;
            layout_page(&mut page, None, self.assets)?;
        }
        let children = std::mem::take(&mut page.children);
        let (cur, next) =
            split::split_nodes(height - pad_bottom, height - pad_top - pad_bottom, children);

        let mut current = page.clone();
        current.children = cur;
        layout_page(
            &mut current,
            Some(height).filter(|h| !h.is_nan()),
            self.assets,
        )?;
        if next.iter().all(Node::is_fixed) {
            return Ok((current, None));
        }
        page.props.shift_remove("bookmark");
        page.children = next;
        layout_page(&mut page, None, self.assets)?;
        Ok((current, Some(page)))
    }

    /// Runs `render` functions once more with the final numbers, and lays the page out again.
    /// The cut stays where it was even if the content grew.
    pub fn finish(&mut self, page: &mut Node, args: &RenderArgs) -> Result<(), String> {
        if has_dynamic(page) {
            self.resolve_dynamic(page, args)?;
            let (height, ..) = page::heights(page);
            let height =
                height.filter(|_| page.props.get("wrap") != Some(&serde_json::Value::Bool(false)));
            layout_page(page, height, self.assets)?;
        }
        Ok(())
    }

    fn resolve_dynamic(&mut self, node: &mut Node, args: &RenderArgs) -> Result<(), String> {
        if let Some(id) = node.render {
            let result = (self.callback)(id, args)?;
            node.set_children(result.children, &self.container);
            node.text = None;
        }
        for child in &mut node.children {
            self.resolve_dynamic(child, args)?;
        }
        Ok(())
    }
}

fn has_dynamic(n: &Node) -> bool {
    n.render.is_some() || n.children.iter().any(has_dynamic)
}
