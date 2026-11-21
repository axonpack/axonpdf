//! Deferred text: each page draws a shared form XObject whose content (the page total) is
//! written at the end, once the value is known. Glyph numbers are fixed at first use and the
//! digits are in every subset, so writing them after the pages is safe.

use pdf_writer::{Content, Name, Ref};

use super::content::PageState;
use super::emit::Out;
use super::{Resources, Writer};
use crate::display::{Deferred, DeferredKey, TextAlign, TextRun};
use crate::fonts::FontStore;

/// Far enough to hold any number at any size.
const REACH: f32 = 1.0e4;

impl Writer {
    pub(super) fn draw_deferred(&mut self, p: &mut PageState, d: &Deferred, out: Out) {
        let r = match self.deferred.iter().find(|(k, _)| same(k, d)) {
            Some((_, r)) => *r,
            None => {
                let r = self.emit.alloc();
                self.deferred.push((*d, r));
                r
            }
        };
        let name = format!("Df{}", r.get());
        Resources::add(&mut p.res.x_objects, name.clone(), r);
        p.save();
        self.set_alpha(p, d.color.a, 1.0, out);
        p.transform([1.0, 0.0, 0.0, 1.0, d.x, d.baseline]);
        p.c.x_object(Name(name.as_bytes()));
        p.restore();
    }

    /// Writes every deferred text now that the values are known. Runs before the fonts.
    pub(super) fn write_deferred(&mut self, fonts: &FontStore, out: Out) {
        let total = self.page_refs.len();
        for (d, r) in std::mem::take(&mut self.deferred) {
            let text = match d.key {
                DeferredKey::TotalPages => total.to_string(),
            };
            let glyphs = fonts.shape(d.font, &text, d.size, 0.0, &[]);
            let width: f32 = glyphs.iter().map(|g| g.x_advance).sum();
            let x = match d.align {
                TextAlign::Left => 0.0,
                TextAlign::Center => (d.width - width) / 2.0,
                TextAlign::Right => d.width - width,
            };
            let run = TextRun {
                font: d.font,
                size: d.size,
                color: d.color,
                x,
                y: 0.0,
                glyphs,
                text,
            };
            let mut c = Content::new();
            c.set_fill_rgb(d.color.r, d.color.g, d.color.b);
            let font_name = self.draw_text(&mut c, &run, fonts);
            let (_, font) = self.font_ref(d.font);
            let data = self.seal(r, c.finish().to_vec());
            self.emit.write(out, |chunk| {
                let mut x = chunk.form_xobject(r, &data);
                x.bbox(pdf_writer::Rect::new(-REACH, -REACH, REACH, REACH));
                x.resources().fonts().pair(Name(font_name.as_bytes()), font);
            });
        }
    }
}

/// Pages share one XObject per deferred value, font, size, color, alignment and box.
fn same(a: &Deferred, b: &Deferred) -> bool {
    let bits = |d: &Deferred| {
        [d.size, d.color.r, d.color.g, d.color.b, d.color.a, d.width].map(f32::to_bits)
    };
    a.key == b.key && a.font == b.font && a.align == b.align && bits(a) == bits(b)
}

pub type Pending = Vec<(Deferred, Ref)>;
