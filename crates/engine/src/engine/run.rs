//! Cutting a document's pages: the first run that counts them when the totals are read, the
//! callbacks the cut makes, and rows fetched as it goes.

use std::cell::Cell;
use std::collections::HashMap;

use super::{RenderArgs, RenderCallback, Visit};
use crate::fonts::FontStore;
use crate::hooks::Hooks;
use crate::layout::Assets;
use crate::layout::remainder::{ROWS, ROWS_DONE};
use crate::paginate::{Finished, Pager, page};
use crate::style::Container;
use crate::tree::node::LAZY;
use crate::tree::{self, Doc, Node, RenderResult};

pub(super) fn layout_pages(
    fonts: &FontStore,
    sizes: &HashMap<u32, (f64, f64)>,
    hooks: &Hooks,
    callback: &mut Option<RenderCallback>,
    doc: Doc,
    visit: &mut Visit,
) -> Result<(), String> {
    hooks.reset();
    let image_size = |id: u32| sizes.get(&id).copied();
    let paint = |id: u32, w: f64, h: f64| hooks.paint(id, w, h);
    let hyphenate = |id: u32, word: &str| hooks.hyphenate(id, word);
    let emoji: Vec<(String, u32)> = doc
        .props
        .get("emoji")
        .and_then(|e| e.as_object())
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| Some((k.clone(), v.get("asset")?.as_u64()? as u32)))
        .collect();
    // A `Rows` batch is styled under the page being cut.
    let more_rows = |n: &mut Node| -> Result<bool, String> {
        // Children waiting as JSON come first, then a batch from the rows' source.
        if !n.pending.is_empty() {
            return Ok(n.build_pending(&PAGE.with(Cell::get), LAZY) > 0);
        }
        if !n.has(ROWS) || n.is(ROWS_DONE) {
            return Ok(false);
        }
        let id = n.number(ROWS).unwrap_or(0.0) as u32;
        let Some(json) = hooks.rows(id) else {
            n.props.insert(ROWS_DONE.into(), true.into());
            return Ok(false);
        };
        let batch: Batch = serde_json::from_str(&json).map_err(|e| format!("rows {id}: {e}"))?;
        if batch.done {
            n.props.insert(ROWS_DONE.into(), true.into());
        }
        let added = !batch.children.is_empty();
        n.append_children(batch.children, &PAGE.with(Cell::get));
        Ok(added)
    };
    let assets = Assets {
        fonts,
        image_size: &image_size,
        paint: &paint,
        hyphenate: &hyphenate,
        hyphenation: doc
            .props
            .get("hyphenationCallback")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
        emoji: &emoji,
        more_rows: &more_rows,
        total_digits: &Cell::new(2),
    };
    let counts = if reads_total(callback, &doc)? {
        Some(run(&assets, callback, &doc, None, None)?)
    } else {
        None
    };
    run(&assets, callback, &doc, counts.as_deref(), Some(visit))?;
    Ok(())
}

/// Whether any `render` function in the tree reads the totals. Each is called once to see.
/// ponytail: functions that only appear inside another's output are not asked; if one of
/// them reads the total it gets `None`. Ask during the cut and restart if that matters.
fn reads_total(callback: &mut Option<RenderCallback>, doc: &Doc) -> Result<bool, String> {
    // Read from each page's JSON without building it: `render` ids (bands' included), page
    // counts, and rows, which are fetched once and cannot be counted ahead.
    // ponytail: Rows documents get no totals; a deferred total drawn at the end would fix it.
    let (mut found, mut count) = (Vec::new(), false);
    for json in doc.page_json() {
        if tree::has_type(json, "ROWS") && !tree::numbers_at(json, ROWS).is_empty() {
            return Ok(false);
        }
        count |= tree::has_type(json, "PAGE_COUNT");
        found.extend(tree::numbers_at(json, "render"));
    }
    if count {
        return Ok(true);
    }
    let args = RenderArgs {
        page_number: 1,
        ..Default::default()
    };
    for id in found {
        if call(callback, id, &args)?.reads_total {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Cuts every `<Page>`, handing each finished page to `visit`. `counts` are the page counts
/// per `<Page>` from a first run, for the totals; without `visit` pages are only counted.
fn run(
    assets: &Assets,
    callback: &mut Option<RenderCallback>,
    doc: &Doc,
    counts: Option<&[u32]>,
    mut visit: Option<&mut Visit>,
) -> Result<Vec<u32>, String> {
    let total = counts.map(|c| c.iter().sum::<u32>());
    let mut call = |id: u32, args: &RenderArgs| call(callback, id, args);
    let mut made = Vec::new();
    let mut number = 1;
    // Rows documents cannot be counted ahead; their total is filled in at the end.
    let pending_total = total.is_none()
        && doc
            .page_json()
            .any(|j| !tree::numbers_at(j, ROWS).is_empty());
    let pages = doc.pages().count();
    for (i, raw) in doc.pages().enumerate() {
        let last_page = i + 1 == pages;
        let (node, container) = page::build(raw?)?;
        PAGE.with(|p| p.set(container));
        let mut pager = Pager {
            assets,
            callback: &mut call,
            container,
            total,
            band_base: page::band_base(&node),
        };
        let sub_total = counts.and_then(|c| c.get(made.len()).copied());
        let mut done = |pager: &mut Pager, mut f: Finished| -> Result<(), String> {
            let Some(visit) = visit.as_mut() else {
                return Ok(());
            };
            let args = RenderArgs {
                page_number: number + f.sub_index,
                total_pages: total,
                sub_page_number: Some(f.sub_index + 1),
                sub_page_total_pages: sub_total,
                is_last: Some(last_page && f.last),
                total_pending: pending_total,
            };
            pager.finish(&mut f.page, &args)?;
            let height = page::heights(&f.page).0.unwrap_or(f.page.layout.height);
            visit(&f.page, container.width, height);
            Ok(())
        };
        let count = pager.paginate(node, number, &mut done)?;
        number += count;
        made.push(count);
    }
    Ok(made)
}

fn call(
    callback: &mut Option<RenderCallback>,
    id: u32,
    args: &RenderArgs,
) -> Result<RenderResult, String> {
    let cb = callback
        .as_mut()
        .ok_or("a render function was used but no render callback is set")?;
    serde_json::from_str(&cb(id, args)).map_err(|e| format!("render callback {id}: {e}"))
}

/// A batch of rows.
#[derive(serde::Deserialize)]
struct Batch {
    #[serde(default)]
    children: Vec<Box<serde_json::value::RawValue>>,
    #[serde(default)]
    done: bool,
}

thread_local! {
    /// The container of the page being cut, for styling rows as they arrive.
    static PAGE: Cell<Container> = Cell::new(Container::default());
}
