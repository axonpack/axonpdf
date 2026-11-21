//! Registered TrueType and OpenType fonts (also from WOFF and WOFF2), shaped with rustybuzz and
//! embedded as subsets.

use rustybuzz::{Direction, Face, Feature, UnicodeBuffer, ttf_parser::Tag};

use super::{FontError, Metrics};
use crate::display::Glyph;

self_cell::self_cell!(
    struct Cell {
        owner: Vec<u8>,
        #[covariant]
        dependent: Face,
    }
);

pub struct Embedded {
    cell: Cell,
    pub metrics: Metrics,
    pub postscript_name: String,
}

impl Embedded {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, FontError> {
        let bytes = match bytes.get(..4) {
            Some(b"wOFF") => wuff::decompress_woff1(&bytes).map_err(|_| FontError::Invalid)?,
            Some(b"wOF2") => wuff::decompress_woff2(&bytes).map_err(|_| FontError::Invalid)?,
            Some(b"ttcf") => return Err(FontError::Collection),
            _ => bytes,
        };
        let cell = Cell::try_new(bytes, |b| Face::from_slice(b, 0).ok_or(FontError::Invalid))?;
        let face = cell.borrow_dependent();
        let hhea = face.tables().hhea;
        let underline = face.underline_metrics();
        let metrics = Metrics {
            units_per_em: face.units_per_em() as f32,
            ascent: hhea.ascender as f32,
            descent: hhea.descender as f32,
            line_gap: hhea.line_gap as f32,
            cap_height: face.capital_height().unwrap_or(hhea.ascender) as f32,
            x_height: face.x_height().unwrap_or(0) as f32,
            underline_position: underline.map_or(0.0, |u| u.position as f32),
            underline_thickness: underline.map_or(0.0, |u| u.thickness as f32),
        };
        let postscript_name = face
            .names()
            .into_iter()
            .find(|n| n.name_id == rustybuzz::ttf_parser::name_id::POST_SCRIPT_NAME)
            .and_then(|n| n.to_string())
            .unwrap_or_else(|| "Font".into());
        Ok(Embedded {
            cell,
            metrics,
            postscript_name,
        })
    }

    pub fn data(&self) -> &[u8] {
        self.cell.borrow_owner()
    }

    pub fn face(&self) -> &Face<'_> {
        self.cell.borrow_dependent()
    }

    pub fn has_char(&self, c: char) -> bool {
        self.face().glyph_index(c).is_some()
    }

    /// The glyph's own advance (hmtx), in font units.
    pub fn advance(&self, glyph: u16) -> u16 {
        self.face()
            .glyph_hor_advance(rustybuzz::ttf_parser::GlyphId(glyph))
            .unwrap_or(0)
    }

    /// Shapes left to right with the font's default features (kerning, ligatures) plus
    /// `features`, each tag turned on or off.
    pub fn shape(
        &self,
        text: &str,
        size: f32,
        letter_spacing: f32,
        features: &[([u8; 4], bool)],
    ) -> Vec<Glyph> {
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        buffer.set_direction(Direction::LeftToRight);
        let features: Vec<Feature> = features
            .iter()
            .map(|&(tag, on)| Feature::new(Tag::from_bytes(&tag), on as u32, ..))
            .collect();
        let out = rustybuzz::shape(self.face(), &features, buffer);
        let scale = size / self.metrics.units_per_em;
        out.glyph_infos()
            .iter()
            .zip(out.glyph_positions())
            .map(|(info, pos)| Glyph {
                id: info.glyph_id as u16,
                cluster: info.cluster,
                x_advance: pos.x_advance as f32 * scale + letter_spacing,
                x_offset: pos.x_offset as f32 * scale,
                y_offset: pos.y_offset as f32 * scale,
            })
            .collect()
    }
}
