//! The engine's public face: fonts and images in, a document tree in, PDF bytes out.

mod job;
mod run;

pub use job::Step;

use run::layout_pages;

use std::collections::HashMap;

use crate::display::ImageId;
use crate::draw;
use crate::fonts::FontStore;
use crate::hooks::{Hooks, HyphenationCallback, PaintCallback, RowsCallback};
use crate::images::ImageStore;
use crate::profile::{self, Phase};
use crate::tree::{self, Doc, Json, Node};
use crate::writer::{Writer, doc_info};

/// What a `render` function is called with. `None` is `undefined` on the adapter's side: while
/// pages are being cut only `page_number` is known.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderArgs {
    pub page_number: u32,
    pub total_pages: Option<u32>,
    pub sub_page_number: Option<u32>,
    pub sub_page_total_pages: Option<u32>,
    /// Whether this is the document's last page; known only in the final call for a page.
    pub is_last: Option<bool>,
    /// The total is filled in when the file is finished (documents with `Rows`, which cannot
    /// be counted ahead): `totalPages` should be [`TOTAL_PAGES`] for the engine to replace.
    pub total_pending: bool,
}

/// Stands for the page total in text until the writer knows it.
pub const TOTAL_PAGES: &str = "\u{E000}T\u{E000}";

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
    /// A resumable render waiting for rows.
    job: Option<job::Job>,
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
            job: None,
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
        self.render_owned(tree_json.to_owned(), out)
    }

    /// [`Engine::render`], reading the JSON in place: the pages' waiting children point into it
    /// instead of holding copies.
    pub fn render_owned(
        &mut self,
        tree_json: String,
        out: &mut dyn FnMut(&[u8]),
    ) -> Result<(), String> {
        let _time = profile::enter(Phase::Layout);
        let text = Json::new(tree_json);
        let doc = parse(&text)?;
        let mut writer = Writer::new(doc_info(&doc.props));
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
            outline: &Default::default(),
        };
        let mut sink = |page: &Node, width: f64, height: f64| {
            let display = paint_page(page, width, height, &cx);
            let _time = profile::enter(Phase::Write);
            writer.write_page(&display, fonts, images, out);
            profile::page();
        };
        layout_pages(fonts, &sizes, hooks, &mut self.callback, doc, &mut sink)?;
        let _time = profile::enter(Phase::Write);
        writer.finish(&self.fonts, out);
        Ok(())
    }

    /// Lays out every page of a document tree, handing each to `visit` with its width and
    /// height as soon as it is final, then dropping it. For checking layout without a PDF.
    pub fn layout(&mut self, tree_json: &str, visit: &mut Visit) -> Result<(), String> {
        let text = Json::new(tree_json.into());
        let doc = tree::parse(&text)?;
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

/// Reads the tree JSON, timed as its own phase.
pub(super) fn parse(tree_json: &Json) -> Result<Doc<'_>, String> {
    let _time = profile::enter(Phase::Parse);
    tree::parse(tree_json)
}

/// A finished page's drawing operations, timed as painting.
pub(super) fn paint_page(
    page: &Node,
    width: f64,
    height: f64,
    cx: &draw::Ctx,
) -> crate::display::PageOut {
    let _time = profile::enter(Phase::Paint);
    draw::page(page, width, height, cx)
}
