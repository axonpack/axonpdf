//! A render that can stop to wait for rows and carry on when they are pushed in, for rows that
//! arrive asynchronously (a database cursor, a stream). It keeps everything a render holds
//! between pages: the writer, the `<Page>` being cut, and where it is in the document.

use std::ops::Range;

use serde_json::{Map, Value};

use super::run::{PAGE, call, count, with_assets};
use super::{Engine, RenderArgs};
use crate::draw;
use crate::hooks::waiting_for;
use crate::layout::remainder::ROWS;
use crate::paginate::{Cut, Pager, page};
use crate::style::Container;
use crate::tree::{self, RawNode};
use crate::writer::{Out, Writer, doc_info};

/// What a resumable render needs next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The file is complete.
    Done,
    /// Push the next batch of `Rows` with this `rows` id, then carry on.
    NeedRows(u32),
}

/// The `<Page>` being cut: its state, page box, and its own padding when it has bands.
type PageCut = (Cut, Container, Option<(f64, f64)>);

pub(super) struct Job {
    json: String,
    /// Where each `PAGE` sits in `json`.
    pages: Vec<Range<usize>>,
    props: Map<String, Value>,
    writer: Writer,
    outline: draw::Outline,
    /// Page counts per `<Page>` from a first run, when the totals are read.
    counts: Option<Vec<u32>>,
    pending_total: bool,
    /// The next `<Page>` to start, and the document page number it starts at.
    next: usize,
    number: u32,
    cut: Option<PageCut>,
}

impl Engine {
    /// Starts a render that stops when it needs rows nobody pushed yet: see
    /// [`Engine::push_rows`]. The bytes made so far go to `out`.
    pub fn render_begin(&mut self, tree_json: String, out: Out) -> Result<Step, String> {
        self.hooks.reset();
        self.hooks.pushed.borrow_mut().clear();
        let doc = tree::parse(&tree_json)?;
        let base = tree_json.as_ptr() as usize;
        #[derive(serde::Deserialize)]
        struct Kind {
            #[serde(rename = "type")]
            kind: String,
        }
        let pages = doc
            .page_json()
            .filter(|j| serde_json::from_str::<Kind>(j).is_ok_and(|k| k.kind == "PAGE"))
            .map(|j| {
                let start = j.as_ptr() as usize - base;
                start..start + j.len()
            })
            .collect();
        let sizes = self.image_sizes();
        let counts = with_assets(&self.fonts, &sizes, &self.hooks, &doc.props, |a| {
            count(a, &mut self.callback, &doc)
        })?;
        let pending_total = counts.is_none()
            && doc
                .page_json()
                .any(|j| !tree::numbers_at(j, ROWS).is_empty());
        let props = doc.props.clone();
        let mut writer = Writer::new(doc_info(&props));
        writer.begin(&mut *out);
        self.job = Some(Job {
            json: tree_json,
            pages,
            props,
            writer,
            outline: Default::default(),
            counts,
            pending_total,
            next: 0,
            number: 1,
            cut: None,
        });
        self.render_step(out)
    }

    /// Hands `Rows` `id` its next batch, `{"children": [...], "done": bool}`, and carries on.
    pub fn push_rows(&mut self, id: u32, batch_json: &str, out: Out) -> Result<Step, String> {
        self.hooks.push_rows(id, batch_json)?;
        self.render_step(out)
    }

    fn render_step(&mut self, out: Out) -> Result<Step, String> {
        let sizes = self.image_sizes();
        let mut job = self.job.take().ok_or("no render has begun")?;
        let (fonts, images, ids, hooks) =
            (&self.fonts, &mut self.images, &self.assets, &self.hooks);
        let callback = &mut self.callback;
        let result = with_assets(fonts, &sizes, hooks, &job.props, |assets| {
            let lookup = |id: u32| Some((*ids.get(&id)?, *sizes.get(&id)?));
            let paint = |id: u32, w: f64, h: f64| hooks.paint(id, w, h);
            let mut call = |id: u32, args: &RenderArgs| call(callback, id, args);
            let total = job.counts.as_ref().map(|c| c.iter().sum::<u32>());
            loop {
                if job.cut.is_none() {
                    let Some(range) = job.pages.get(job.next).cloned() else {
                        return Ok(Step::Done);
                    };
                    let raw: RawNode =
                        serde_json::from_str(&job.json[range]).map_err(|e| e.to_string())?;
                    let (node, container) = page::build(raw)?;
                    let band_base = page::band_base(&node);
                    let pager = Pager {
                        assets,
                        callback: &mut call,
                        container,
                        total,
                        band_base,
                    };
                    job.cut = Some((pager.begin(node, job.number), container, band_base));
                }
                let (cut, container, band_base) = job.cut.as_mut().unwrap();
                PAGE.with(|p| p.set(*container));
                let mut pager = Pager {
                    assets,
                    callback: &mut call,
                    container: *container,
                    total,
                    band_base: *band_base,
                };
                let finished = match pager.next(cut) {
                    Err(e) => return waiting_for(&e).map(Step::NeedRows).ok_or(e),
                    Ok(f) => f,
                };
                let Some(mut f) = finished else {
                    job.number += Pager::count(cut);
                    job.next += 1;
                    job.cut = None;
                    continue;
                };
                let sub_total = job.counts.as_ref().and_then(|c| c.get(job.next).copied());
                let args = RenderArgs {
                    page_number: job.number + f.sub_index,
                    total_pages: total,
                    sub_page_number: Some(f.sub_index + 1),
                    sub_page_total_pages: sub_total,
                    is_last: Some(job.next + 1 == job.pages.len() && f.last),
                    total_pending: job.pending_total,
                };
                pager.finish(&mut f.page, &args)?;
                let height = page::heights(&f.page).0.unwrap_or(f.page.layout.height);
                let cx = draw::Ctx {
                    fonts,
                    images: &lookup,
                    paint: &paint,
                    forms: Default::default(),
                    outline: &job.outline,
                };
                let display = draw::page(&f.page, container.width, height, &cx);
                drop(f);
                job.writer.write_page(&display, fonts, images, &mut *out);
            }
        });
        match &result {
            Ok(Step::Done) => job.writer.finish(fonts, out),
            Ok(_) => self.job = Some(job),
            Err(_) => {}
        }
        result
    }
}
