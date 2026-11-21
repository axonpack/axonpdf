//! The engine's public face: fonts and images in, a document tree in, PDF bytes out.

use std::cell::Cell;
use std::collections::HashMap;

use crate::display::ImageId;
use crate::draw;
use crate::fonts::FontStore;
use crate::hooks::{Hooks, HyphenationCallback, PaintCallback, RowsCallback};
use crate::images::ImageStore;
use crate::layout::Assets;
use crate::layout::remainder::{ROWS, ROWS_DONE};
use crate::paginate::{Finished, Pager, page};
use crate::style::Container;
use crate::tree::{self, Doc, Node, RawNode, RenderResult};
use crate::writer::{DocInfo, Writer};

/// What a `render` function is called with. `None` is `undefined` on the adapter's side: while
/// pages are being cut only `page_number` is known.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderArgs {
    pub page_number: u32,
    pub total_pages: Option<u32>,
    pub sub_page_number: Option<u32>,
    pub sub_page_total_pages: Option<u32>,
}

/// Takes a finished page with its width and height.
pub type Visit<'a> = dyn FnMut(&Node, f64, f64) + 'a;

/// Calls a `render` function by id; answers `{"children": [...], "readsTotal": bool}`.
pub type RenderCallback = Box<dyn FnMut(u32, &RenderArgs) -> String>;

pub struct Engine {
    fonts: FontStore,
    images: ImageStore,
    /// The document's asset ids to stored images.
    assets: HashMap<u32, ImageId>,
    callback: Option<RenderCallback>,
    hooks: Hooks,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            fonts: FontStore::new(),
            images: ImageStore::new(),
            assets: HashMap::new(),
            callback: None,
            hooks: Hooks::default(),
        }
    }

    pub fn add_font(
        &mut self,
        family: &str,
        weight: u16,
        italic: bool,
        bytes: Vec<u8>,
    ) -> Result<(), String> {
        self.fonts
            .add_font(family, weight, italic, bytes)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn add_image(&mut self, id: u32, bytes: Vec<u8>) -> Result<(), String> {
        let image = self.images.add(&bytes).map_err(|e| format!("{e:?}"))?;
        self.assets.insert(id, image);
        Ok(())
    }

    pub fn set_render_callback(&mut self, cb: RenderCallback) {
        self.callback = Some(cb);
    }

    /// Canvas `paint` functions, by the `paint` id on `CANVAS` nodes.
    pub fn set_paint_callback(&mut self, cb: PaintCallback) {
        *self.hooks.paint.get_mut() = Some(cb);
    }

    /// `Rows` batches, by the `rows` id on `ROWS` nodes. The engine asks for the next batch when
    /// the rows it has run out before the next cut, so only a few batches are held at a time.
    pub fn set_rows_callback(&mut self, cb: RowsCallback) {
        *self.hooks.rows.get_mut() = Some(cb);
    }

    /// Hyphenation functions, by the `hyphenationCallback` id on `TEXT` and `DOCUMENT` props.
    /// The built-in split they may fall back to is [`crate::text::hyphen::builtin`].
    pub fn set_hyphenation_callback(&mut self, cb: HyphenationCallback) {
        *self.hooks.hyphenate.get_mut() = Some(cb);
    }

    /// Renders a document tree (see [`crate::tree`]), sending the PDF to `out` in order as
    /// pages finish. When a `render` function reads the page total, pages are cut once to count
    /// them, then again to draw them.
    pub fn render(&mut self, tree_json: &str, out: &mut dyn FnMut(&[u8])) -> Result<(), String> {
        let doc = tree::parse(tree_json)?;
        let mut writer = Writer::new(doc_info(&doc));
        writer.begin(out);
        let sizes = self.image_sizes();
        let (fonts, images, assets) = (&self.fonts, &mut self.images, &self.assets);
        let lookup = |id: u32| Some((*assets.get(&id)?, *sizes.get(&id)?));
        let hooks = &self.hooks;
        let paint = |id: u32, w: f64, h: f64| hooks.paint(id, w, h);
        let cx = draw::Ctx {
            fonts,
            images: &lookup,
            paint: &paint,
            forms: Default::default(),
        };
        let mut sink = |page: &Node, width: f64, height: f64| {
            let display = draw::page(page, width, height, &cx);
            writer.write_page(&display, fonts, images, out);
        };
        layout_pages(fonts, &sizes, hooks, &mut self.callback, doc, &mut sink)?;
        writer.finish(&self.fonts, out);
        Ok(())
    }

    /// Lays out every page of a document tree, handing each to `visit` with its width and
    /// height as soon as it is final, then dropping it. For checking layout without a PDF.
    pub fn layout(&mut self, tree_json: &str, visit: &mut Visit) -> Result<(), String> {
        let doc = tree::parse(tree_json)?;
        let sizes = self.image_sizes();
        layout_pages(
            &self.fonts,
            &sizes,
            &self.hooks,
            &mut self.callback,
            doc,
            visit,
        )
    }

    /// Pixel sizes of the images, by the document's asset ids.
    fn image_sizes(&self) -> HashMap<u32, (f64, f64)> {
        self.assets
            .iter()
            .map(|(&k, &v)| {
                let (w, h) = self.images.size(v);
                (k, (w as f64, h as f64))
            })
            .collect()
    }
}

fn layout_pages(
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
    // A page count is drawn with the total, so it needs the count too.
    fn ids(n: &RawNode, out: &mut Vec<u32>, count: &mut bool) {
        out.extend(n.render);
        *count |= n.kind == "PAGE_COUNT";
        n.children.iter().for_each(|c| ids(c, out, count));
        // A page's bands: a function's id, or a node.
        for band in ["header", "footer"].iter().filter_map(|k| n.props.get(*k)) {
            match serde_json::from_value::<RawNode>(band.clone()) {
                Ok(b) => ids(&b, out, count),
                Err(_) => out.extend(
                    band.get("render")
                        .and_then(|r| r.as_u64())
                        .map(|r| r as u32),
                ),
            }
        }
    }
    let (mut found, mut count) = (Vec::new(), false);
    for page in doc.pages() {
        ids(&page?, &mut found, &mut count);
    }
    // Rows are fetched once and cannot be counted ahead.
    // ponytail: their documents get no totals; a deferred total drawn at the end would fix it.
    if doc.pages().any(|p| p.is_ok_and(|p| has_rows(&p))) {
        return Ok(false);
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
    for raw in doc.pages() {
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

/// The document's metadata props.
fn doc_info(doc: &Doc) -> DocInfo {
    let text = |k: &str| {
        doc.props
            .get(k)
            .and_then(|v| v.as_str())
            .map(str::to_string)
    };
    DocInfo {
        title: text("title"),
        author: text("author"),
        subject: text("subject"),
        keywords: text("keywords"),
        creator: text("creator"),
        producer: text("producer"),
        language: text("language"),
        ..Default::default()
    }
}

fn has_rows(n: &RawNode) -> bool {
    (n.kind == "ROWS" && n.props.contains_key(ROWS)) || n.children.iter().any(has_rows)
}

/// A batch of rows.
#[derive(serde::Deserialize)]
struct Batch {
    #[serde(default)]
    children: Vec<RawNode>,
    #[serde(default)]
    done: bool,
}

thread_local! {
    /// The container of the page being cut, for styling rows as they arrive.
    static PAGE: Cell<Container> = Cell::new(Container::default());
}
