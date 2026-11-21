//! Making one page: the template for it, the room it leaves, what fills that room, and the page
//! laid out with it.

use serde_json::json;

use super::{Cut, build, fetch_all, fill, flow, items, template};
use crate::RenderArgs;
use crate::layout::layout_page;
use crate::paginate::{Finished, Pager, page};
use crate::tree::{Node, RawNode, truthy};

impl Cut {
    /// The template for page `n`, rendered, and the way down to its `SLOT`.
    fn chrome(
        &self,
        pager: &mut Pager,
        args: &RenderArgs,
    ) -> Result<(Vec<Node>, Vec<usize>), String> {
        let c = pager.container;
        let Some(id) = self.layout else {
            let slot = template::node(json!({"type": template::SLOT}), &c, &self.shell);
            return Ok((vec![slot], vec![0]));
        };
        let made = (pager.callback)(id, args)?;
        let mut nodes: Vec<Node> = made
            .children
            .iter()
            .map(|j| Node::build(RawNode::parse(j), &c, Some((&self.shell.style, false))))
            .collect();
        for n in &mut nodes {
            fetch_all(n, pager)?;
            pager.resolve_dynamic(n, args)?;
            settle(n);
        }
        let mut first = None;
        template::check(template::find(&nodes, &mut Vec::new(), &mut first))?;
        Ok((nodes, first.unwrap()))
    }

    /// The next page, or `None` when the content is all placed.
    pub fn next(&mut self, pager: &mut Pager) -> Result<Option<Finished>, String> {
        self.start(pager)?;
        if self.frags.is_empty() {
            return Ok(None);
        }
        let n = self.count + 1;
        let args = self.numbers.args(n, pager.total);
        let (chrome, path) = self.chrome(pager, &args)?;
        let (height, ..) = page::heights(&self.shell);
        let wrap = self.shell.props.get("wrap").is_none_or(truthy);

        let mut fake = self.shell.clone();
        fake.children = chrome.clone();
        template::put(
            &mut fake.children,
            &path,
            vec![template::probe(&pager.container, &self.shell)],
        );
        layout_page(&mut fake, height, pager.assets)?;
        let (.., width, room) = template::region(&fake, &path);
        let width_then = *self.width.get_or_insert(width);
        if (width - width_then).abs() > 0.001 {
            return Err(format!(
                "[layout] The page layout changes the content width on page {n} ({width} vs {width_then}). Chrome may vary in height per page, not width."
            ));
        }
        // A page without a height has no ceiling, like one that does not wrap.
        let room = match height {
            Some(_) if wrap && room <= 0.0 => {
                return Err(format!(
                    "[layout] The page layout leaves no room for content on page {n}."
                ));
            }
            Some(_) if wrap => room,
            _ => f64::INFINITY,
        };

        let (numbers, total) = (self.numbers, pager.total);
        let mut mx = |node: &Node, absolute: bool, sub: u32| -> Result<items::Item, String> {
            let m = measure(pager, node, &numbers.args(sub, total))?;
            Ok(if absolute {
                flow::marker(m)
            } else {
                flow::to_item(m)
            })
        };
        let result = fill::fill(&mut self.frags, room, n, true, false, &mut mx)?;
        self.frags = result.remaining;

        let mut page = self.shell.clone();
        if n > 1 {
            page.props.shift_remove("bookmark");
        }
        page.children = chrome;
        template::put(&mut page.children, &path, build::nodes(&result.placed));
        layout_page(&mut page, height, pager.assets)?;
        self.count = n;
        Ok(Some(Finished {
            page,
            sub_index: n - 1,
            last: self.frags.is_empty(),
        }))
    }
}

/// No more `render` functions to call in a node: they have been called for its page.
fn settle(n: &mut Node) {
    n.dynamic = false;
    n.children.iter_mut().for_each(settle);
}

/// Renders a lazy node for its page and lays it out alone at the width it first had.
fn measure(pager: &mut Pager, node: &Node, args: &RenderArgs) -> Result<Node, String> {
    let mut n = node.clone();
    pager.resolve_dynamic(&mut n, args)?;
    settle(&mut n);
    let l = n.layout;
    let width = if l.width != 0.0 {
        l.width
    } else {
        pager.container.width
    } + l.margin[1]
        + l.margin[3];
    let raw = json!({"type": "PAGE", "style": {"width": width}});
    let mut fake = Node::build(serde_json::from_value(raw).unwrap(), &pager.container, None);
    fake.children = vec![n];
    layout_page(&mut fake, None, pager.assets)?;
    let mut m = fake.children.pop().unwrap();
    m.layout.top = 0.0;
    Ok(m)
}
