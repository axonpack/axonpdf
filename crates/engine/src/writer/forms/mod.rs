//! AcroForm fields: each field is a widget annotation on its page; field sets are parent
//! fields written at the end with their kids. Viewers draw the field appearances
//! (`NeedAppearances`), except checkboxes, which carry their own.

mod field;

use std::collections::HashMap;

use pdf_writer::{Content, Name, Ref, Str};

use super::Writer;
use super::emit::Out;
use super::strings::text;
use crate::display::FontId;

#[derive(Default)]
pub struct Form {
    /// Top-level fields and sets.
    fields: Vec<Ref>,
    /// Field sets by id: object, name, parent and kids.
    sets: HashMap<u32, (Ref, String, Option<Ref>, Vec<Ref>)>,
    zapf: Option<Ref>,
    /// Checkbox appearances by (character, width bits, height bits).
    looks: HashMap<(u8, u32, u32), Ref>,
}

impl Writer {
    pub(super) fn add_field_set(&mut self, id: u32, name: &str, parent: Option<u32>) {
        let r = self.emit.alloc();
        let parent_ref = self.attach(parent, r);
        self.form
            .sets
            .insert(id, (r, name.to_string(), parent_ref, Vec::new()));
    }

    /// Lists `child` under its set, or at the top; returns the set's object.
    fn attach(&mut self, parent: Option<u32>, child: Ref) -> Option<Ref> {
        match parent.and_then(|p| self.form.sets.get_mut(&p)) {
            Some(set) => {
                set.3.push(child);
                Some(set.0)
            }
            None => {
                self.form.fields.push(child);
                None
            }
        }
    }

    /// A ZapfDingbats character centred low in the box, the checkbox's on or off look.
    fn check_look(&mut self, ch: u8, w: f32, h: f32, out: Out) -> Ref {
        let key = (ch, w.to_bits(), h.to_bits());
        if let Some(r) = self.form.looks.get(&key) {
            return *r;
        }
        let zapf = *self.form.zapf.get_or_insert_with(|| self.emit.alloc());
        let mut c = Content::new();
        c.begin_marked_content(Name(b"Tx"));
        c.save_state();
        c.set_font(Name(b"ZaDb"), h * 0.8);
        c.begin_text();
        c.next_line(w * 0.45, h / 4.0);
        c.show(Str(&[ch]));
        c.end_text();
        c.restore_state();
        c.end_marked_content();
        let r = self.emit.alloc();
        let data = self.seal(r, c.finish().to_vec());
        self.emit.write(out, |chunk| {
            let mut x = chunk.form_xobject(r, &data);
            x.bbox(pdf_writer::Rect::new(0.0, 0.0, w, h));
            x.resources().fonts().pair(Name(b"ZaDb"), zapf);
        });
        self.form.looks.insert(key, r);
        r
    }

    /// The AcroForm dictionary and the field sets, if the document has fields.
    pub(super) fn write_form(&mut self, out: Out) -> Option<Ref> {
        if self.form.fields.is_empty() {
            return None;
        }
        let (helv_name, helv) = self.font_ref(FontId(0));
        let form = self.emit.alloc();
        let sets: Vec<_> = self.form.sets.drain().map(|(_, v)| v).collect();
        for (r, name, parent, kids) in sets {
            let name = self.seal(r, text(&name));
            self.emit.write(out, |c| {
                let mut d = c.indirect(r).dict();
                d.pair(Name(b"T"), Str(&name));
                if let Some(p) = parent {
                    d.pair(Name(b"Parent"), p);
                }
                d.insert(Name(b"Kids")).array().items(kids.iter().copied());
            });
        }
        if let Some(zapf) = self.form.zapf {
            self.emit.write(out, |c| {
                c.type1_font(zapf).base_font(Name(b"ZapfDingbats"));
            });
        }
        let da = self.seal(form, format!("/{helv_name} 0 Tf 0 g").into_bytes());
        let (fields, zapf) = (std::mem::take(&mut self.form.fields), self.form.zapf);
        self.emit.write(out, |c| {
            let mut d = c.indirect(form).dict();
            d.insert(Name(b"Fields"))
                .array()
                .items(fields.iter().copied());
            d.pair(Name(b"NeedAppearances"), true);
            d.pair(Name(b"DA"), Str(&da));
            let mut dr = d.insert(Name(b"DR")).dict();
            let mut fonts = dr.insert(Name(b"Font")).dict();
            fonts.pair(Name(helv_name.as_bytes()), helv);
            if let Some(z) = zapf {
                fonts.pair(Name(b"ZaDb"), z);
            }
        });
        Some(form)
    }
}
