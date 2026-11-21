//! Fonts the pages used: the Standard 14 by name, registered fonts as subset CID fonts written
//! at the end. A glyph's number in the subset is fixed the first time a page uses it.

use std::collections::HashMap;

use pdf_writer::types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Filter, Name, Rect, Ref, Str};
use rustybuzz::ttf_parser::Tag;
use subsetter::GlyphRemapper;

use super::Writer;
use super::emit::Out;
use crate::display::FontId;
use crate::fonts::{Embedded, Font, FontStore, standard};

#[derive(Default)]
pub struct Used(HashMap<FontId, Entry>);

struct Entry {
    r: Ref,
    remap: GlyphRemapper,
    /// Text of each subset glyph, for copy and search.
    unicode: HashMap<u16, String>,
}

impl Writer {
    /// The resource name and object of a font, reserved on first use.
    pub(super) fn font_ref(&mut self, id: FontId) -> (String, Ref) {
        let emit = &mut self.emit;
        let entry = self.fonts.0.entry(id).or_insert_with(|| Entry {
            r: emit.alloc(),
            remap: GlyphRemapper::new(),
            unicode: HashMap::new(),
        });
        (format!("F{}", id.0), entry.r)
    }

    /// The subset glyph number of a registered font's glyph, given once and kept.
    pub(super) fn subset_glyph(&mut self, id: FontId, glyph: u16, text: &str) -> u16 {
        let entry = self.fonts.0.get_mut(&id).expect("font_ref first");
        let new = entry.remap.remap(glyph);
        if !text.is_empty() {
            entry.unicode.entry(new).or_insert_with(|| text.to_string());
        }
        new
    }

    pub(super) fn write_fonts(&mut self, store: &FontStore, out: Out) {
        let mut used: Vec<(FontId, Entry)> =
            std::mem::take(&mut self.fonts.0).into_iter().collect();
        used.sort_by_key(|(id, _)| id.0);
        for (id, mut entry) in used {
            match store.font(id) {
                Font::Standard(i) => {
                    let name = standard::NAMES[*i];
                    self.emit.write(out, |c| {
                        let mut f = c.type1_font(entry.r);
                        f.base_font(Name(name.as_bytes()));
                        if *i < 12 {
                            f.encoding_predefined(Name(b"WinAnsiEncoding"));
                        }
                    });
                }
                Font::Embedded(e) => {
                    // Digits are always in the subset, for numbers drawn after the pages.
                    for d in '0'..='9' {
                        if let Some(g) = e.face().glyph_index(d) {
                            let new = entry.remap.remap(g.0);
                            entry.unicode.entry(new).or_insert_with(|| d.to_string());
                        }
                    }
                    self.write_embedded(e, &entry, out);
                }
            }
        }
    }

    fn write_embedded(&mut self, e: &Embedded, entry: &Entry, out: Out) {
        let (cid, descriptor, file, cmap) = (
            self.emit.alloc(),
            self.emit.alloc(),
            self.emit.alloc(),
            self.emit.alloc(),
        );
        let face = e.face();
        let cff = face.tables().cff.is_some();
        let subset = subsetter::subset(e.data(), 0, &entry.remap).ok();
        let program = subset.as_deref().and_then(|s| {
            if !cff {
                return Some(s.to_vec());
            }
            let raw = rustybuzz::ttf_parser::RawFace::parse(s, 0).ok()?;
            raw.table(Tag::from_bytes(b"CFF ")).map(<[u8]>::to_vec)
        });
        let scale = 1000.0 / e.metrics.units_per_em;
        let base = format!(
            "{}+{}",
            tag(&entry.remap),
            e.postscript_name.replace(
                |c: char| !c.is_ascii_graphic() || "()<>[]{}/%#".contains(c),
                ""
            )
        );
        let widths: Vec<f32> = entry
            .remap
            .remapped_gids()
            .map(|g| e.advance(g) as f32 * scale)
            .collect();
        let bbox = face.global_bounding_box();
        let m = e.metrics;
        let italic = face.italic_angle();

        let r = entry.r;
        self.emit.write(out, |c| {
            c.type0_font(r)
                .base_font(Name(base.as_bytes()))
                .encoding_predefined(Name(b"Identity-H"))
                .descendant_font(cid)
                .to_unicode(cmap);
        });
        // Strings in an encrypted file are encrypted too.
        let (registry, ordering) = (
            self.seal(cid, b"Adobe".to_vec()),
            self.seal(cid, b"Identity".to_vec()),
        );
        self.emit.write(out, |c| {
            let mut f = c.cid_font(cid);
            f.subtype(if cff {
                CidFontType::Type0
            } else {
                CidFontType::Type2
            });
            f.base_font(Name(base.as_bytes()));
            f.system_info(SystemInfo {
                registry: Str(&registry),
                ordering: Str(&ordering),
                supplement: 0,
            });
            f.font_descriptor(descriptor);
            f.widths().consecutive(0, widths.iter().copied());
            if !cff {
                f.cid_to_gid_map_predefined(Name(b"Identity"));
            }
        });
        self.emit.write(out, |c| {
            let mut flags = FontFlags::SYMBOLIC;
            flags.set(FontFlags::ITALIC, italic != 0.0);
            let mut d = c.font_descriptor(descriptor);
            d.name(Name(base.as_bytes()))
                .flags(flags)
                .italic_angle(italic);
            d.bbox(Rect::new(
                bbox.x_min as f32 * scale,
                bbox.y_min as f32 * scale,
                bbox.x_max as f32 * scale,
                bbox.y_max as f32 * scale,
            ));
            d.ascent(m.ascent * scale)
                .descent(m.descent * scale)
                .cap_height(m.cap_height * scale)
                .stem_v(80.0);
            if program.is_some() {
                if cff {
                    d.font_file3(file)
                } else {
                    d.font_file2(file)
                };
            }
        });
        let program = program.unwrap_or_default();
        let compressed = self.seal(
            file,
            miniz_oxide::deflate::compress_to_vec_zlib(&program, 6),
        );
        self.emit.write(out, |c| {
            let mut s = c.stream(file, &compressed);
            s.filter(Filter::FlateDecode);
            if cff {
                s.pair(Name(b"Subtype"), Name(b"CIDFontType0C"));
            } else {
                s.pair(Name(b"Length1"), program.len() as i32);
            }
        });
        let mut map = UnicodeCmap::new(
            Name(b"Custom"),
            SystemInfo {
                registry: Str(b"Adobe"),
                ordering: Str(b"UCS"),
                supplement: 0,
            },
        );
        let mut pairs: Vec<_> = entry.unicode.iter().collect();
        pairs.sort();
        for (g, s) in pairs {
            map.pair_with_multiple(*g, s.chars());
        }
        let cmap_data = self.seal(
            cmap,
            miniz_oxide::deflate::compress_to_vec_zlib(&map.finish(), 6),
        );
        self.emit.write(out, |c| {
            c.stream(cmap, &cmap_data).filter(Filter::FlateDecode);
        });
    }
}

/// Six capitals from the glyph set, the usual prefix that marks a subset.
fn tag(remap: &GlyphRemapper) -> String {
    let mut h: u32 = 2166136261;
    for g in remap.remapped_gids() {
        h = (h ^ g as u32).wrapping_mul(16777619);
    }
    (0..6)
        .map(|i| (b'A' + ((h >> (i * 5)) % 26) as u8) as char)
        .collect()
}
