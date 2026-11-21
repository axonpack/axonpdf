//! The Standard 14 fonts: Adobe's AFM metrics, never embedded, written with WinAnsiEncoding.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::{Metrics, winansi};
use crate::display::Glyph;

/// The 14 PostScript names, in [`crate::display::FontId`] order.
pub const NAMES: [&str; 14] = [
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-Oblique",
    "Helvetica-BoldOblique",
    "Times-Roman",
    "Times-Bold",
    "Times-Italic",
    "Times-BoldItalic",
    "Courier",
    "Courier-Bold",
    "Courier-Oblique",
    "Courier-BoldOblique",
    "Symbol",
    "ZapfDingbats",
];

// ponytail: the AFM text (~640 KB, ~120 KB gzipped) is parsed on first use; precompute tables
// at build time if the WASM size matters.
macro_rules! afm {
    ($($name:literal),*) => { [$(include_str!(concat!("../../../../vendor/afm/", $name, ".afm"))),*] };
}
const AFM: [&str; 14] = afm!(
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-Oblique",
    "Helvetica-BoldOblique",
    "Times-Roman",
    "Times-Bold",
    "Times-Italic",
    "Times-BoldItalic",
    "Courier",
    "Courier-Bold",
    "Courier-Oblique",
    "Courier-BoldOblique",
    "Symbol",
    "ZapfDingbats"
);

pub struct Standard {
    pub name: &'static str,
    /// Advance by WinAnsi code, in 1/1000 em.
    pub widths: [u16; 256],
    kerns: HashMap<(u8, u8), i16>,
    pub metrics: Metrics,
    /// The AFM's own ascender (1/1000 em), which canvas text sits under.
    pub afm_ascender: f32,
}

/// The parsed font at `index` into [`NAMES`].
pub fn get(index: usize) -> &'static Standard {
    static CACHE: [OnceLock<Standard>; 14] = [const { OnceLock::new() }; 14];
    CACHE[index].get_or_init(|| parse(NAMES[index], AFM[index]))
}

impl Standard {
    pub fn has_char(&self, c: char) -> bool {
        winansi::name(winansi::code(c)) != ".notdef"
    }

    /// One glyph per character, kerned with the next one in `text`.
    pub fn shape(&self, text: &str, size: f32, letter_spacing: f32) -> Vec<Glyph> {
        let scale = size / 1000.0;
        let codes: Vec<(u16, usize)> = text
            .char_indices()
            .map(|(i, c)| (winansi::code(c), i))
            .collect();
        let width = |code: u16| self.widths.get(code as usize).copied().unwrap_or(0) as f32;
        codes
            .iter()
            .enumerate()
            .map(|(k, &(code, cluster))| {
                let kern = codes
                    .get(k + 1)
                    .map_or(0.0, |&(next, _)| self.kern(code, next));
                Glyph {
                    id: code,
                    cluster: cluster as u32,
                    x_advance: (width(code) + kern) * scale + letter_spacing,
                    x_offset: 0.0,
                    y_offset: 0.0,
                }
            })
            .collect()
    }

    fn kern(&self, left: u16, right: u16) -> f32 {
        if left > 255 || right > 255 {
            return 0.0;
        }
        self.kerns
            .get(&(left as u8, right as u8))
            .copied()
            .unwrap_or(0) as f32
    }
}

fn parse(name: &'static str, afm: &str) -> Standard {
    let mut by_name: HashMap<&str, u16> = HashMap::new();
    let mut kern_lines = Vec::new();
    let mut afm_ascender = 0.0;
    for line in afm.lines() {
        if let Some(v) = line.strip_prefix("Ascender ") {
            afm_ascender = v.trim().parse().unwrap_or(0.0);
        } else if let Some(rest) = line.strip_prefix("C ") {
            let field = |key: &str| rest.split(';').find_map(|f| f.trim().strip_prefix(key));
            if let (Some(wx), Some(n)) = (field("WX "), field("N ")) {
                by_name.insert(n.trim(), wx.trim().parse().unwrap_or(0));
            }
        } else if let Some(rest) = line.strip_prefix("KPX ") {
            kern_lines.push(rest);
        }
    }
    let mut widths = [0u16; 256];
    let mut codes_of: HashMap<&str, Vec<u8>> = HashMap::new();
    for (code, glyph) in winansi::NAMES.iter().enumerate() {
        if *glyph != ".notdef" {
            widths[code] = by_name.get(glyph).copied().unwrap_or(0);
            codes_of.entry(glyph).or_default().push(code as u8);
        }
    }
    let mut kerns = HashMap::new();
    for rest in kern_lines {
        let mut parts = rest.split_whitespace();
        let (Some(l), Some(r), Some(v)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let (Some(ls), Some(rs), Ok(v)) = (codes_of.get(l), codes_of.get(r), v.parse::<i16>())
        else {
            continue;
        };
        for &a in ls {
            for &b in rs {
                kerns.insert((a, b), v);
            }
        }
    }
    Standard {
        name,
        widths,
        kerns,
        metrics: metrics(name),
        afm_ascender,
    }
}

/// Fixed metrics the layout has always used for these fonts, not the AFM's own values.
fn metrics(name: &str) -> Metrics {
    let (descent, cap_height, x_height) = match name.split('-').next() {
        Some("Times") => (-220.0, 650.0, 440.0),
        Some("Courier") => (-230.0, 550.0, 390.0),
        _ => (-200.0, 690.0, 490.0),
    };
    Metrics {
        units_per_em: 1000.0,
        ascent: 900.0,
        descent,
        line_gap: 0.0,
        cap_height,
        x_height,
        underline_position: -100.0,
        underline_thickness: 50.0,
    }
}
