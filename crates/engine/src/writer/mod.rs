//! The streamed PDF writer. `begin` sends the header, every page leaves as soon as it is
//! written, and `finish` writes what can only be known at the end: fonts, page tree, outline,
//! destinations, info, cross-reference table and trailer. Only small per-page records stay.

mod border;
mod content;
mod crypt;
mod deferred;
mod emit;
mod fonts;
mod forms;
mod gradient;
mod images;
mod info;
mod nav;
mod pdfa;
mod props;
mod security;
mod state;
mod strings;
mod svg_image;
mod text;

use std::collections::HashMap;

use pdf_writer::{Chunk, Filter, Finish, Name, Rect as PdfRect, Ref};

use crate::display::{ImageId, PageOut};
use crate::fonts::FontStore;
use crate::images::ImageStore;
pub use crypt::{Encryption, Permissions, Printing};
use emit::Emitter;
pub use emit::Out;
pub use info::{DocInfo, PageLayout, PageMode, PdfVersion};
pub use pdfa::Conformance;
pub use props::doc_info;
pub use strings::Date;

pub struct Writer {
    info: DocInfo,
    emit: Emitter,
    catalog: Ref,
    pages: Ref,
    page_refs: Vec<Ref>,
    fonts: fonts::Used,
    images: HashMap<ImageId, Ref>,
    /// The size in points of each SVG image's form, by object.
    svg_sizes: HashMap<Ref, (f32, f32)>,
    /// Shared ExtGState objects by (fill alpha, stroke alpha) bits.
    alphas: HashMap<(u32, u32), Ref>,
    gradients: gradient::Written,
    nav: nav::Nav,
    form: forms::Form,
    /// Deferred texts the pages drew, written at the end.
    deferred: deferred::Pending,
    /// The file identifier, and the encryption keyed to it.
    id: [u8; 16],
    crypt: Option<crypt::Crypt>,
}

impl Writer {
    pub fn new(mut info: DocInfo) -> Self {
        // PDF/A-1 is PDF 1.4; the later parts are 1.7.
        if let (Some(c), PdfVersion::V1_3) = (info.conformance, info.pdf_version) {
            info.pdf_version = if c.part == 1 {
                PdfVersion::V1_4
            } else {
                PdfVersion::V1_7
            };
        }
        let mut emit = Emitter::default();
        let catalog = emit.alloc();
        let pages = emit.alloc();
        let id = security::file_id(&info);
        let crypt = info
            .encryption
            .as_ref()
            .map(|e| crypt::Crypt::new(e, info.pdf_version, &id));
        Writer {
            info,
            emit,
            catalog,
            pages,
            page_refs: Vec::new(),
            fonts: Default::default(),
            images: HashMap::new(),
            svg_sizes: HashMap::new(),
            alphas: HashMap::new(),
            gradients: Default::default(),
            nav: Default::default(),
            form: Default::default(),
            deferred: Vec::new(),
            id,
            crypt,
        }
    }

    /// The file header.
    pub fn begin(&mut self, out: Out) {
        let header = format!(
            "%PDF-{}\n%\u{FF}\u{FF}\u{FF}\u{FF}\n",
            self.info.pdf_version.as_str()
        );
        // The marker bytes must be raw 0xFF, not UTF-8.
        let bytes: Vec<u8> = header.chars().map(|c| c as u32 as u8).collect();
        self.emit.raw(out, &bytes);
    }

    /// Writes one page and everything it needs that is not written yet (images, annotations).
    pub fn write_page(
        &mut self,
        page: &PageOut,
        fonts: &FontStore,
        images: &mut ImageStore,
        out: Out,
    ) {
        let page_ref = self.emit.alloc();
        let parts = self.content(page, page_ref, fonts, images, out);
        let content_ref = self.emit.alloc();
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&parts.content, 6);
        let data = self.seal(content_ref, compressed);
        self.emit.write(out, |c| {
            c.stream(content_ref, &data).filter(Filter::FlateDecode);
        });
        let pages = self.pages;
        self.emit.write(out, |c: &mut Chunk| {
            let mut p = c.page(page_ref);
            p.parent(pages)
                .media_box(PdfRect::new(0.0, 0.0, page.width, page.height));
            p.contents(content_ref);
            if !parts.annots.is_empty() {
                p.annotations(parts.annots.iter().copied());
            }
            parts.resources.write(p.resources());
        });
        self.page_refs.push(page_ref);
    }

    /// Writes the fonts, page tree, outline, destinations, catalog, info and trailer.
    pub fn finish(mut self, fonts: &FontStore, out: Out) {
        // The form names its font, so it goes before the fonts are written.
        let form = self.write_form(out);
        self.write_deferred(fonts, out);
        self.write_fonts(fonts, out);
        let (pages, kids) = (self.pages, std::mem::take(&mut self.page_refs));
        self.emit.write(out, |c| {
            c.pages(pages)
                .count(kids.len() as i32)
                .kids(kids.iter().copied());
        });
        let mut nav = self.write_nav(&kids, out);
        nav.form = form;
        nav.pdfa = self.write_pdfa(out);
        let info = self.write_info(out);
        self.write_catalog(nav, out);
        let encrypt = self.write_encrypt(out);
        let trailer = self.trailer(info, encrypt);
        self.emit.end(out, trailer.as_bytes());
    }

    /// Encrypts a string or stream of object `id` when the document is encrypted.
    fn seal(&self, id: Ref, data: Vec<u8>) -> Vec<u8> {
        match &self.crypt {
            Some(c) => c.seal(id.get(), &data),
            None => data,
        }
    }

    /// The shared graphics state that sets these alphas.
    fn alpha_state(&mut self, fill: f32, stroke: f32, out: Out) -> (String, Ref) {
        let key = (fill.to_bits(), stroke.to_bits());
        let r = match self.alphas.get(&key) {
            Some(r) => *r,
            None => {
                let r = self.emit.alloc();
                self.emit.write(out, |c| {
                    c.ext_graphics(r)
                        .non_stroking_alpha(fill)
                        .stroking_alpha(stroke);
                });
                self.alphas.insert(key, r);
                r
            }
        };
        (format!("Gs{}", r.get()), r)
    }
}

/// Resources one page uses, by name.
#[derive(Default)]
struct Resources {
    fonts: Vec<(String, Ref)>,
    x_objects: Vec<(String, Ref)>,
    ext_g_states: Vec<(String, Ref)>,
    patterns: Vec<(String, Ref)>,
}

impl Resources {
    fn add(list: &mut Vec<(String, Ref)>, name: String, r: Ref) {
        if !list.iter().any(|(n, _)| *n == name) {
            list.push((name, r));
        }
    }

    fn write(&self, mut res: pdf_writer::writers::Resources) {
        let groups = [
            &self.fonts,
            &self.x_objects,
            &self.ext_g_states,
            &self.patterns,
        ];
        let keys: [&[u8]; 4] = [b"Font", b"XObject", b"ExtGState", b"Pattern"];
        for (list, key) in groups.into_iter().zip(keys) {
            if !list.is_empty() {
                let mut dict = res.insert(Name(key)).dict();
                for (name, r) in list {
                    dict.pair(Name(name.as_bytes()), *r);
                }
                dict.finish();
            }
        }
    }
}
