//! Cutting a document's pages: the first run that counts them when the totals are read, the
//! callbacks the cut makes, and rows fetched as it goes.

use std::cell::Cell;
use std::collections::HashMap;

use super::{RenderArgs, RenderCallback, Visit};
use crate::fonts::FontStore;
use crate::hooks::Hooks;
use crate::layout::Assets;
use crate::layout::remainder::{LAZY_CHILDREN, ROWS, ROWS_DONE, batch_id};
use crate::paginate::{self, Finished, Pager, next, page};
use crate::profile::{self, Phase};
use crate::style::Container;
use crate::tree::node::LAZY;
use crate::tree::{self, Doc, Json, Node, RawNode, RenderResult};
use serde_json::{Map, Value};

/// Builds what layout needs from the engine's stores and the document's props, for `f`.
pub(super) fn with_assets<R>(
    fonts: &FontStore,
    sizes: &HashMap<u32, (f64, f64)>,
    hooks: &Hooks,
    props: &Map<String, Value>,
    f: impl FnOnce(&Assets) -> R,
) -> R {
    let image_size = |id: u32| sizes.get(&id).copied();
    let paint = |id: u32, w: f64, h: f64| hooks.paint(id, w, h);
    let hyphenate = |id: u32, word: &str| hooks.hyphenate(id, word);
    let emoji: Vec<(String, u32)> = props
        .get("emoji")
        .and_then(|e| e.as_object())
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| Some((k.clone(), v.get("asset")?.as_u64()? as u32)))
        .collect();
    let batch = |id: u32| {
        let (children, done) = hooks.rows(id)?;
        Ok((children.into_iter().map(Json::from).collect(), done))
    };
    // Children waiting as JSON come first, then a batch from the rows' source; both are styled
    // under the page being cut.
    let more_rows = |n: &mut Node| -> Result<bool, String> {
        if !n.pending.is_empty() {
            return Ok(n.build_pending(&PAGE.with(Cell::get), LAZY) > 0);
        }
        let Some(id) = batch_id(n).filter(|_| !n.is(ROWS_DONE)) else {
            return Ok(false);
        };
        let (children, done) = hooks.rows(id)?;
        if done {
            n.props.insert(ROWS_DONE.into(), true.into());
        }
        let added = !children.is_empty();
        n.append_children(children, &PAGE.with(Cell::get));
        Ok(added)
    };
    let assets = Assets {
        fonts,
        image_size: &image_size,
        paint: &paint,
        hyphenate: &hyphenate,
        hyphenation: props
            .get("hyphenationCallback")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
        emoji: &emoji,
        more_rows: &more_rows,
        batch: &batch,
        total_digits: &Cell::new(2),
    };
    f(&assets)
}

pub(super) fn layout_pages(
    fonts: &FontStore,
    sizes: &HashMap<u32, (f64, f64)>,
    hooks: &Hooks,
    callback: &mut Option<RenderCallback>,
    doc: Doc,
    visit: &mut Visit,
) -> Result<(), String> {
    hooks.reset();
    with_assets(fonts, sizes, hooks, &doc.props, |assets| {
        let counts = count(hooks, assets, callback, &doc)?;
        run(assets, callback, &doc, counts.as_deref(), Some(visit))?;
        Ok(())
    })
}

/// The page counts per `<Page>`, from a first run, when something reads the total.
pub(super) fn count(
    hooks: &Hooks,
    assets: &Assets,
    callback: &mut Option<RenderCallback>,
    doc: &Doc,
) -> Result<Option<Vec<u32>>, String> {
    if !reads_total(callback, doc)? {
        return Ok(None);
    }
    // Lazy children fetched while counting are replayed for the pass that draws the pages.
    hooks.record(true);
    let counts = run(assets, callback, doc, None, None);
    hooks.record(false);
    Ok(Some(counts?))
}

/// Whether any `render` function in the tree reads the totals. Each is called once to see.
/// ponytail: functions that only appear inside another's output are not asked; if one of
/// them reads the total it gets `None`. Ask during the cut and restart if that matters.
fn reads_total(callback: &mut Option<RenderCallback>, doc: &Doc) -> Result<bool, String> {
    // Read from each page's JSON without building it: `render` ids (bands' included), page
    // counts, and rows, which are fetched once and cannot be counted ahead.
    // ponytail: Rows documents get no totals; a deferred total drawn at the end would fix it.
    // The adapter asks for a count when lazy content it has seen reads the total.
    if doc.props.get("countPages").is_some_and(tree::truthy) {
        return Ok(true);
    }
    let (mut found, mut count) = (Vec::new(), false);
    for json in doc.page_json() {
        if tree::has_type(json, "ROWS") && !tree::numbers_at(json, ROWS).is_empty() {
            return Ok(false);
        }
        count |= tree::has_type(json, "PAGE_COUNT");
        found.extend(tree::numbers_at(json, "render"));
        found.extend(next::Head::read(json).layout);
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
    // Rows documents cannot be counted ahead; their total is filled in at the end. So is that of
    // an uncounted document with lazy content, which may hold something that reads it.
    let pending_total = total.is_none() && doc.page_json().any(deferred_total);
    let pages = doc.pages().count();
    let newer = doc.page_json().any(|j| next::Head::read(j).wants_next());
    let page_list = doc.page_json().filter_map(|j| match doc.read(j) {
        Ok(n) if n.kind != "PAGE" => None,
        r => Some((r, j)),
    });
    for (i, (raw, json)) in page_list.enumerate() {
        let last_page = i + 1 == pages;
        let sub_total = counts.and_then(|c| c.get(made.len()).copied());
        let totals = (sub_total, pending_total);
        let mut raw = raw?;
        fill_page(&mut raw, assets)?;
        let (cut, container, band_base) =
            paginate::open(raw, json, newer, number, totals, &mut call)?;
        PAGE.with(|p| p.set(container));
        let mut pager = Pager {
            assets,
            callback: &mut call,
            container,
            total,
            band_base,
        };
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
        let count = pager.paginate(cut, &mut done)?;
        number += count;
        made.push(count);
    }
    Ok(made)
}

pub(super) fn call(
    callback: &mut Option<RenderCallback>,
    id: u32,
    args: &RenderArgs,
) -> Result<RenderResult, String> {
    let _time = profile::enter(Phase::Callbacks);
    let cb = callback
        .as_mut()
        .ok_or("a render function was used but no render callback is set")?;
    serde_json::from_str(&cb(id, args)).map_err(|e| format!("render callback {id}: {e}"))
}

thread_local! {
    /// The container of the page being cut, for styling rows as they arrive.
    pub(super) static PAGE: Cell<Container> = Cell::new(Container::default());
}

/// A page sent without its children (`"lazy": id`, no `children`) gets them all from the adapter
/// when it is opened, so it is cut exactly as if they had been inline.
pub(super) fn fill_page(raw: &mut RawNode, a: &Assets) -> Result<(), String> {
    let lazy = raw.props.get(LAZY_CHILDREN).and_then(|v| v.as_u64());
    let Some(id) = lazy.filter(|_| raw.kind == "PAGE" && raw.children.is_empty()) else {
        return Ok(());
    };
    raw.props.shift_remove(LAZY_CHILDREN);
    loop {
        let (children, done) = (a.batch)(id as u32)?;
        raw.children.extend(children);
        if done {
            return Ok(());
        }
    }
}

/// Whether a page's total, when it was not counted, is left for the writer: the page has rows,
/// or lazy content that may read it.
pub(super) fn deferred_total(page_json: &str) -> bool {
    [ROWS, LAZY_CHILDREN]
        .iter()
        .any(|key| !tree::numbers_at(page_json, key).is_empty())
}
