//! Fonts: the Standard 14 by metrics, registered TTF/OTF/WOFF/WOFF2 from bytes, family lookup
//! and shaping.

mod embedded;
mod error;
mod family;
pub mod standard;
pub mod winansi;

use std::collections::HashMap;

pub use crate::display::FontId;
use crate::display::Glyph;
pub use embedded::Embedded;
pub use error::FontError;
use family::Source;
pub use family::weight_from_name;

/// Vertical metrics in font units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub units_per_em: f32,
    pub ascent: f32,
    /// Negative below the baseline.
    pub descent: f32,
    pub line_gap: f32,
    pub cap_height: f32,
    pub x_height: f32,
    pub underline_position: f32,
    pub underline_thickness: f32,
}

pub enum Font {
    /// Index into [`standard::NAMES`].
    Standard(usize),
    Embedded(Embedded),
}

/// Every font of a document. Ids 0 to 13 are the Standard 14, in [`standard::NAMES`] order.
pub struct FontStore {
    fonts: Vec<Font>,
    families: HashMap<String, Vec<Source>>,
}

impl Default for FontStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FontStore {
    /// The built-in families: `Helvetica`, `Courier` and `Times-Roman` with bold and italic faces,
    /// and each bold or italic face under its own name. `Symbol` and `ZapfDingbats` need
    /// [`FontStore::add_standard`].
    pub fn new() -> Self {
        let mut store = FontStore {
            fonts: (0..14).map(Font::Standard).collect(),
            families: HashMap::new(),
        };
        for (family, base) in [("Helvetica", 0), ("Times-Roman", 4), ("Courier", 8)] {
            for (i, (weight, italic)) in [(400, false), (700, false), (400, true), (700, true)]
                .into_iter()
                .enumerate()
            {
                store.register(family, weight, italic, FontId(base + i as u32));
            }
        }
        for (i, name) in standard::NAMES[..12].iter().enumerate() {
            if !store.families.contains_key(*name) {
                store.register(name, 400, false, FontId(i as u32));
            }
        }
        store
    }

    /// Registers a TTF, OTF, WOFF or WOFF2 file as one face of `family`.
    pub fn add_font(
        &mut self,
        family: &str,
        weight: u16,
        italic: bool,
        bytes: Vec<u8>,
    ) -> Result<FontId, FontError> {
        let id = FontId(self.fonts.len() as u32);
        self.fonts.push(Font::Embedded(Embedded::parse(bytes)?));
        self.register(family, weight, italic, id);
        Ok(id)
    }

    /// Registers a Standard 14 font, by PostScript name, as one face of `family`.
    pub fn add_standard(
        &mut self,
        family: &str,
        weight: u16,
        italic: bool,
        name: &str,
    ) -> Result<FontId, FontError> {
        let index = standard::NAMES.iter().position(|n| *n == name);
        let id = FontId(index.ok_or_else(|| FontError::UnknownStandard(name.into()))? as u32);
        self.register(family, weight, italic, id);
        Ok(id)
    }

    fn register(&mut self, family: &str, weight: u16, italic: bool, font: FontId) {
        self.families
            .entry(family.into())
            .or_default()
            .push(Source {
                weight,
                italic,
                font,
            });
    }

    /// The face of `family` nearest to `weight` in the asked style. Callers try each family of a
    /// `fontFamily` list and end with `Helvetica`.
    pub fn resolve(&self, family: &str, weight: u16, italic: bool) -> Result<FontId, FontError> {
        let sources = self
            .families
            .get(family)
            .ok_or_else(|| FontError::NotRegistered(family.into()))?;
        family::pick(sources, weight, italic).ok_or_else(|| FontError::NoMatch {
            family: family.into(),
            weight,
            italic,
        })
    }

    pub fn font(&self, id: FontId) -> &Font {
        &self.fonts[id.0 as usize]
    }

    pub fn metrics(&self, id: FontId) -> Metrics {
        match self.font(id) {
            Font::Standard(i) => standard::get(*i).metrics,
            Font::Embedded(e) => e.metrics,
        }
    }

    /// Whether the font draws `c` (for per-character fallback).
    pub fn has_char(&self, id: FontId, c: char) -> bool {
        match self.font(id) {
            Font::Standard(i) => standard::get(*i).has_char(c),
            Font::Embedded(e) => e.has_char(c),
        }
    }

    /// Shapes `text` at `size` points. Each glyph's advance includes kerning and
    /// `letter_spacing` (the last glyph's too). `features` turns OpenType features on or off;
    /// the Standard 14 ignore them.
    pub fn shape(
        &self,
        id: FontId,
        text: &str,
        size: f32,
        letter_spacing: f32,
        features: &[([u8; 4], bool)],
    ) -> Vec<Glyph> {
        match self.font(id) {
            Font::Standard(i) => standard::get(*i).shape(text, size, letter_spacing),
            Font::Embedded(e) => e.shape(text, size, letter_spacing, features),
        }
    }

    /// The width of `text` in points, as [`FontStore::shape`] lays it out.
    pub fn measure(&self, id: FontId, text: &str, size: f32, letter_spacing: f32) -> f32 {
        self.shape(id, text, size, letter_spacing, &[])
            .iter()
            .map(|g| g.x_advance)
            .sum()
    }

    /// A glyph's own advance in font units, without kerning or spacing.
    pub fn advance(&self, id: FontId, glyph: u16) -> f32 {
        match self.font(id) {
            Font::Standard(i) => standard::get(*i)
                .widths
                .get(glyph as usize)
                .copied()
                .unwrap_or(0) as f32,
            Font::Embedded(e) => e.advance(glyph) as f32,
        }
    }
}
