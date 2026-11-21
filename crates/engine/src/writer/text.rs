//! Text runs as `TJ` arrays: glyph codes with an adjustment wherever a glyph's advance differs
//! from its own width (kerning, letter spacing, justification).

use pdf_writer::{Content, Name, Str};

use super::Writer;
use crate::display::TextRun;
use crate::fonts::{Font, FontStore};

impl Writer {
    /// Draws a run with the fill color already set. The page space is flipped (y down), so the
    /// text matrix flips glyphs back upright.
    pub(super) fn draw_text(
        &mut self,
        c: &mut Content,
        run: &TextRun,
        fonts: &FontStore,
    ) -> String {
        let (name, _) = self.font_ref(run.font);
        let standard = matches!(fonts.font(run.font), Font::Standard(_));
        let upem = fonts.metrics(run.font).units_per_em;
        let pieces = self.encode(run, standard);

        c.begin_text();
        c.set_font(Name(name.as_bytes()), run.size);
        let mut pen = run.x;
        let mut moved = false;
        c.set_text_matrix([1.0, 0.0, 0.0, -1.0, run.x, run.y]);
        let mut pending: Vec<(Vec<u8>, f32)> = Vec::new();
        for (glyph, code) in run.glyphs.iter().zip(pieces) {
            let natural = if code.is_empty() {
                0.0
            } else {
                fonts.advance(run.font, glyph.id) * run.size / upem
            };
            if glyph.x_offset != 0.0 || glyph.y_offset != 0.0 {
                flush(c, &mut pending);
                c.set_text_matrix([
                    1.0,
                    0.0,
                    0.0,
                    -1.0,
                    pen + glyph.x_offset,
                    run.y - glyph.y_offset,
                ]);
                c.show(Str(&code));
                moved = true;
            } else {
                if moved {
                    flush(c, &mut pending);
                    c.set_text_matrix([1.0, 0.0, 0.0, -1.0, pen, run.y]);
                    moved = false;
                }
                // Thousandths of an em, positive moves left.
                let adjust = (natural - glyph.x_advance) * 1000.0 / run.size;
                pending.push((code, if adjust.abs() > 1e-4 { adjust } else { 0.0 }));
            }
            pen += glyph.x_advance;
        }
        flush(c, &mut pending);
        c.end_text();
        name
    }

    /// Each glyph's bytes: one WinAnsi byte for the Standard 14 (nothing past 255), else its
    /// two-byte subset number.
    fn encode(&mut self, run: &TextRun, standard: bool) -> Vec<Vec<u8>> {
        if standard {
            return run
                .glyphs
                .iter()
                .map(|g| if g.id < 256 { vec![g.id as u8] } else { vec![] })
                .collect();
        }
        let ends = cluster_ends(run);
        run.glyphs
            .iter()
            .zip(ends)
            .map(|(g, end)| {
                let start = g.cluster as usize;
                let text = run.text.get(start..end).unwrap_or("");
                self.subset_glyph(run.font, g.id, text)
                    .to_be_bytes()
                    .to_vec()
            })
            .collect()
    }
}

/// Where each glyph's text ends: at the next larger cluster. Glyphs after the first one of a
/// cluster get an empty range, so the text is copied once.
fn cluster_ends(run: &TextRun) -> Vec<usize> {
    let mut sorted: Vec<u32> = run.glyphs.iter().map(|g| g.cluster).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let mut seen = std::collections::HashSet::new();
    run.glyphs
        .iter()
        .map(|g| {
            if !seen.insert(g.cluster) {
                return g.cluster as usize;
            }
            let i = sorted.partition_point(|&c| c <= g.cluster);
            sorted.get(i).map_or(run.text.len(), |&c| c as usize)
        })
        .collect()
}

fn flush(c: &mut Content, pending: &mut Vec<(Vec<u8>, f32)>) {
    if pending.is_empty() {
        return;
    }
    let mut tj = c.show_positioned();
    let mut items = tj.items();
    let mut bytes = Vec::new();
    for (code, adjust) in pending.drain(..) {
        bytes.extend(code);
        if adjust != 0.0 {
            items.show(Str(&bytes));
            items.adjust(adjust);
            bytes.clear();
        }
    }
    if !bytes.is_empty() {
        items.show(Str(&bytes));
    }
}
