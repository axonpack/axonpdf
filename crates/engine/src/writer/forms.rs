//! AcroForm fields: each field is a widget annotation on its page; field sets are parent
//! fields written at the end with their kids. Viewers draw the field appearances
//! (`NeedAppearances`), except checkboxes, which carry their own.

use std::collections::HashMap;

use pdf_writer::{Content, Finish, Name, Ref, Str};

use super::emit::Out;
use super::strings::text;
use super::{Writer, nav};
use crate::display::{Align, Field, FieldFormat, FieldKind, FontId};

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

const READ_ONLY: i32 = 1;
const REQUIRED: i32 = 2;
const NO_EXPORT: i32 = 4;
const MULTILINE: i32 = 0x1000;
const PASSWORD: i32 = 0x2000;
const COMBO: i32 = 0x20000;
const EDIT: i32 = 0x40000;
const SORT: i32 = 0x80000;
const MULTI_SELECT: i32 = 0x200000;
const NO_SPELL: i32 = 0x400000;

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

    /// Writes the widget for a field at `rect` (`[x0 y0 x1 y1]` in PDF space).
    pub(super) fn field(&mut self, f: &Field, rect: [f32; 4], out: Out) -> Ref {
        let (font_name, _) = self.font_ref(FontId(0));
        let id = self.emit.alloc();
        let parent = self.attach(f.parent, id);
        let mut flags = flag(f.read_only, READ_ONLY)
            | flag(f.required, REQUIRED)
            | flag(f.no_export, NO_EXPORT);
        let looks = match &f.kind {
            FieldKind::Checkbox {
                checked,
                on_state,
                off_state,
                x_mark,
                ..
            } => {
                let (w, h) = (rect[2] - rect[0], rect[3] - rect[1]);
                let on = self.check_look(if *x_mark { b'8' } else { b'4' }, w, h, out);
                let off = self.check_look(if *x_mark { b' ' } else { b'8' }, w, h, out);
                let state = if *checked { on_state } else { off_state };
                Some((on_state.clone(), on, off_state.clone(), off, state.clone()))
            }
            _ => None,
        };
        let seal = |w: &Self, s: &str| w.seal(id, text(s));
        let name = seal(self, &f.name);
        let (value, default) = (
            f.value.as_deref().map(|v| seal(self, v)),
            f.default_value.as_deref().map(|v| seal(self, v)),
        );
        let options: Vec<Vec<u8>> = match &f.kind {
            FieldKind::Choice { options, .. } => options.iter().map(|o| seal(self, o)).collect(),
            _ => Vec::new(),
        };
        let script = match &f.kind {
            FieldKind::Text {
                format: Some(format),
                ..
            } => {
                let (keystroke, show) = scripts(format);
                Some((seal(self, &keystroke), seal(self, &show)))
            }
            _ => None,
        };
        let da = match &f.kind {
            FieldKind::Text {
                font_size: Some(size),
                ..
            } => Some(seal(self, &format!("/{font_name} {size} Tf 0 g"))),
            _ => None,
        };
        self.emit.write(out, |c| {
            let mut d = c.indirect(id).dict();
            nav::annot_base(&mut d, b"Widget", rect);
            d.pair(Name(b"F"), 4);
            d.insert(Name(b"C")).array().items([0.0f32; 3]);
            d.pair(Name(b"T"), Str(&name));
            if let Some(p) = parent {
                d.pair(Name(b"Parent"), p);
            }
            match &f.kind {
                FieldKind::Text {
                    align,
                    multiline,
                    password,
                    no_spell,
                    max_len,
                    ..
                } => {
                    d.pair(Name(b"FT"), Name(b"Tx"));
                    flags |= flag(*multiline, MULTILINE)
                        | flag(*password, PASSWORD)
                        | flag(*no_spell, NO_SPELL);
                    match align {
                        Align::Left => {}
                        Align::Center => {
                            d.pair(Name(b"Q"), 1);
                        }
                        Align::Right => {
                            d.pair(Name(b"Q"), 2);
                        }
                    }
                    if let Some(n) = max_len {
                        d.pair(Name(b"MaxLen"), *n as i32);
                    }
                }
                FieldKind::Checkbox {
                    background, border, ..
                } => {
                    d.pair(Name(b"FT"), Name(b"Btn"));
                    if background.is_some() || border.is_some() {
                        let mut mk = d.insert(Name(b"MK")).dict();
                        for (key, color) in [(b"BG", background), (b"BC", border)] {
                            if let Some(c) = color {
                                mk.insert(Name(key)).array().items([c.r, c.g, c.b]);
                            }
                        }
                    }
                }
                FieldKind::Choice {
                    combo,
                    sort,
                    edit,
                    multi_select,
                    no_spell,
                    ..
                } => {
                    d.pair(Name(b"FT"), Name(b"Ch"));
                    flags |= flag(*combo, COMBO) | flag(*sort, SORT) | flag(*edit, EDIT);
                    flags |= flag(*multi_select, MULTI_SELECT) | flag(*no_spell, NO_SPELL);
                    d.insert(Name(b"Opt"))
                        .array()
                        .items(options.iter().map(|o| Str(o)));
                }
            }
            if flags != 0 {
                d.pair(Name(b"Ff"), flags);
            }
            if let Some((on_name, on, off_name, off, state)) = &looks {
                d.pair(Name(b"V"), Name(state.as_bytes()))
                    .pair(Name(b"DV"), Name(state.as_bytes()));
                d.pair(Name(b"AS"), Name(state.as_bytes()));
                let mut ap = d.insert(Name(b"AP")).dict();
                for key in [b"N", b"D"] {
                    ap.insert(Name(key))
                        .dict()
                        .pair(Name(on_name.as_bytes()), *on)
                        .pair(Name(off_name.as_bytes()), *off);
                }
                ap.finish();
            } else {
                if let Some(v) = &value {
                    d.pair(Name(b"V"), Str(v));
                }
                if let Some(v) = &default {
                    d.pair(Name(b"DV"), Str(v));
                }
            }
            if let Some(da) = &da {
                d.pair(Name(b"DA"), Str(da));
            }
            if let Some((keystroke, show)) = &script {
                let mut aa = d.insert(Name(b"AA")).dict();
                for (key, js) in [(b"K", keystroke), (b"F", show)] {
                    aa.insert(Name(key))
                        .dict()
                        .pair(Name(b"S"), Name(b"JavaScript"))
                        .pair(Name(b"JS"), Str(js));
                }
            }
        });
        id
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

fn flag(on: bool, bit: i32) -> i32 {
    if on { bit } else { 0 }
}

/// The viewer's keystroke and format scripts for a text field format.
fn scripts(f: &FieldFormat) -> (String, String) {
    let (name, params) = match f {
        FieldFormat::Zip => ("Special", "0".to_string()),
        FieldFormat::ZipPlus4 => ("Special", "1".to_string()),
        FieldFormat::Phone => ("Special", "2".to_string()),
        FieldFormat::Ssn => ("Special", "3".to_string()),
        FieldFormat::Date(pattern) => ("DateEx", format!("\"{pattern}\"")),
        FieldFormat::Time(style) => ("Time", style.to_string()),
        FieldFormat::Number {
            decimals,
            sep_comma,
            neg_style,
            currency,
            currency_prepend,
        } => (
            "Number",
            format!(
                "{decimals},{},\"{neg_style}\",null,\"{currency}\",{currency_prepend}",
                if *sep_comma { 0 } else { 1 }
            ),
        ),
        FieldFormat::Percent {
            decimals,
            sep_comma,
        } => (
            "Percent",
            format!("{decimals},{}", if *sep_comma { 0 } else { 1 }),
        ),
    };
    let (keystroke, format) = match name.strip_suffix("Ex") {
        Some(base) => (
            format!("AF{base}_KeystrokeEx"),
            format!("AF{base}_FormatEx"),
        ),
        None => (format!("AF{name}_Keystroke"), format!("AF{name}_Format")),
    };
    (
        format!("{keystroke}({params});"),
        format!("{format}({params});"),
    )
}
