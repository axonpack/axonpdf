//! The engine's public face: fonts and images in, a document tree in, PDF bytes out.

use std::collections::HashMap;

use crate::display::ImageId;
use crate::fonts::FontStore;
use crate::images::ImageStore;
use crate::layout::Assets;
use crate::paginate::{Finished, Pager, page};
use crate::tree::{self, RawNode, RenderResult};
use crate::writer::{DocInfo, Out, Writer};
use crate::draw;

/// What a `render` function is called with. `None` is `undefined` on the adapter's side: while
/// pages are being cut only `page_number` is known.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderArgs {
    pub page_number: u32,
    pub total_pages: Option<u32>,
    pub sub_page_number: Option<u32>,
    pub sub_page_total_pages: Option<u32>,
}

/// Calls a `render` function by id; answers `{"children": [...], "readsTotal": bool}`.
pub type RenderCallback = Box<dyn FnMut(u32, &RenderArgs) -> String>;

pub struct Engine {
    fonts: FontStore,
    images: ImageStore,
    /// The document's asset ids to stored images.
    assets: HashMap<u32, ImageId>,
    callback: Option<RenderCallback>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Engine { fonts: FontStore::new(), images: ImageStore::new(), assets: HashMap::new(), callback: None }
    }

    pub fn add_font(&mut self, family: &str, weight: u16, italic: bool, bytes: Vec<u8>) -> Result<(), String> {
        self.fonts.add_font(family, weight, italic, bytes).map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn add_image(&mut self, id: u32, bytes: Vec<u8>) -> Result<(), String> {
        let image = self.images.add(&bytes).map_err(|e| format!("{e:?}"))?;
        self.assets.insert(id, image);
        Ok(())
    }

    pub fn set_render_callback(&mut self, cb: RenderCallback) {
        self.callback = Some(cb);
    }

    /// Renders a document tree (see [`crate::tree`]), sending the PDF to `out` in order as
    /// pages finish. When a `render` function reads the page total, pages are cut once to count
    /// them, then again to draw them.
    pub fn render(&mut self, tree_json: &str, out: &mut dyn FnMut(&[u8])) -> Result<(), String> {
        let doc = tree::parse(tree_json)?;
        let counts = if self.reads_total(&doc)? { Some(self.run(tree::parse(tree_json)?, None, None)?) } else { None };
        self.run(doc, counts.as_deref(), Some(out))?;
        Ok(())
    }

    /// Whether any `render` function in the tree reads the totals. Each is called once to see.
    /// ponytail: functions that only appear inside another's output are not asked; if one of
    /// them reads the total it gets `None`. Ask during the cut and restart if that matters.
    fn reads_total(&mut self, doc: &RawNode) -> Result<bool, String> {
        fn ids(n: &RawNode, out: &mut Vec<u32>) {
            out.extend(n.render);
            n.children.iter().for_each(|c| ids(c, out));
        }
        let mut found = Vec::new();
        ids(doc, &mut found);
        let args = RenderArgs { page_number: 1, ..Default::default() };
        for id in found {
            if self.call(id, &args)?.reads_total {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn call(&mut self, id: u32, args: &RenderArgs) -> Result<RenderResult, String> {
        let cb = self.callback.as_mut().ok_or("a render function was used but no render callback is set")?;
        serde_json::from_str(&cb(id, args)).map_err(|e| format!("render callback {id}: {e}"))
    }

    /// Cuts every `<Page>` and, with `out`, draws and writes each page as it is cut. `counts`
    /// are the page counts per `<Page>` from a first run, for the totals. Returns the counts.
    fn run(&mut self, doc: RawNode, counts: Option<&[u32]>, out: Option<Out>) -> Result<Vec<u32>, String> {
        let sizes: HashMap<u32, (f64, f64)> = self
            .assets
            .iter()
            .map(|(&k, &v)| {
                let (w, h) = self.images.size(v);
                (k, (w as f64, h as f64))
            })
            .collect();
        let image_size = |id: u32| sizes.get(&id).copied();
        let assets = Assets { fonts: &self.fonts, image_size: &image_size };
        let total = counts.map(|c| c.iter().sum::<u32>());
        let mut writer = out.is_some().then(|| Writer::new(doc_info(&doc)));
        let mut out = out;
        if let (Some(w), Some(o)) = (writer.as_mut(), out.as_mut()) {
            w.begin(*o);
        }
        let callback = &mut self.callback;
        let mut call = |id: u32, args: &RenderArgs| -> Result<RenderResult, String> {
            let cb = callback.as_mut().ok_or("a render function was used but no render callback is set")?;
            serde_json::from_str(&cb(id, args)).map_err(|e| format!("render callback {id}: {e}"))
        };
        let lookup = |id: u32| self.assets.get(&id).copied();
        let mut made = Vec::new();
        let mut number = 1;
        for raw in doc.children.into_iter().filter(|r| r.kind == "PAGE") {
            let (node, container) = page::build(raw)?;
            let mut pager = Pager { assets: &assets, callback: &mut call, container };
            let sub_total = counts.and_then(|c| c.get(made.len()).copied());
            let mut done = |pager: &mut Pager, mut f: Finished| -> Result<(), String> {
                let (Some(w), Some(o)) = (writer.as_mut(), out.as_mut()) else { return Ok(()) };
                let args = RenderArgs {
                    page_number: number + f.sub_index,
                    total_pages: total,
                    sub_page_number: Some(f.sub_index + 1),
                    sub_page_total_pages: sub_total,
                };
                pager.finish(&mut f.page, &args)?;
                let height = page::heights(&f.page).0.unwrap_or(f.page.layout.height);
                let display = draw::page(&f.page, container.width, height, &lookup);
                drop(f);
                w.write_page(&display, &self.fonts, &mut self.images, *o);
                Ok(())
            };
            let count = pager.paginate(node, number, &mut done)?;
            number += count;
            made.push(count);
        }
        if let (Some(w), Some(o)) = (writer, out) {
            w.finish(&self.fonts, o);
        }
        Ok(made)
    }
}

/// The document's metadata props.
fn doc_info(doc: &RawNode) -> DocInfo {
    let text = |k: &str| doc.props.get(k).and_then(|v| v.as_str()).map(str::to_string);
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
