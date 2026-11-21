//! Text: a `Text` node's runs are prepared once (fonts picked, glyphs shaped, words split into
//! syllables), then broken into lines for whatever width layout asks for.

pub mod breaker;
mod finalize;
pub mod hyphen;
mod lines;
mod prepare;

pub use lines::layout;
pub use prepare::{Fragment, prepare};

use std::rc::Rc;

use crate::display::{Color, FontId};

/// What a stretch of text looks like. Shared by every run cut from the same fragment.
#[derive(Clone, Debug, PartialEq)]
pub struct Attrs {
    pub size: f64,
    pub color: Color,
    pub opacity: Option<f64>,
    pub letter_spacing: f64,
    pub line_height: Option<f64>,
    pub underline: bool,
    pub strike: bool,
    pub decoration_color: Color,
    pub decoration_style: String,
    pub link: Option<String>,
    /// `left`, `right`, `center` or `justify`.
    pub align: String,
    pub background: Option<Color>,
    pub indent: f64,
    /// `sub` or `super`.
    pub vertical_align: Option<String>,
    /// An image drawn in place of each U+FFFC in the run (inline images and emoji).
    pub attachment: Option<Attachment>,
}

/// An image on a text line. `y_offset` moves it down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attachment {
    pub width: f64,
    pub height: f64,
    pub y_offset: f64,
    pub asset: u32,
}

/// One shaped glyph. `index` is the character it starts at; lengths are points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub id: u16,
    pub index: usize,
    /// The character after the last one the glyph draws (more than one for a ligature).
    pub end: usize,
    pub advance: f64,
    pub x_offset: f64,
    pub y_offset: f64,
    pub whitespace: bool,
}

/// Characters `start..end` of a paragraph in one font and one style.
#[derive(Clone, Debug)]
pub struct Run {
    pub start: usize,
    pub end: usize,
    pub attrs: Rc<Attrs>,
    pub font: FontId,
    /// Ascent and descent (negative) in points, and the line gap.
    pub ascent: f64,
    pub descent: f64,
    pub line_gap: f64,
    pub glyphs: Vec<Glyph>,
    /// The glyphs drawn for a hyphen added at a break, and for an ellipsis.
    pub hyphen: Glyph,
    pub ellipsis: Glyph,
    /// Ligatures cut by a line break, shaped again on each side of every possible cut:
    /// `(ligature start, cut, glyphs before, glyphs after)`.
    pub splits: Vec<(usize, usize, Vec<Glyph>, Vec<Glyph>)>,
}

impl Run {
    /// The height a line gives this run: its line height, else ascent to descent plus gap.
    pub fn height(&self) -> f64 {
        match self.attrs.line_height {
            Some(h) if h != 0.0 => h,
            _ => self.line_gap + self.ascent - self.descent,
        }
    }
}

/// One paragraph (text up to and including a newline), ready to break.
#[derive(Clone, Debug, Default)]
pub struct Paragraph {
    pub chars: Vec<char>,
    pub runs: Vec<Run>,
    /// Character ranges of the syllables: words, hyphenation pieces and runs of spaces.
    pub syllables: Vec<(usize, usize)>,
}

/// A run's share of a line.
#[derive(Clone, Debug)]
pub struct LineRun {
    pub attrs: Rc<Attrs>,
    /// The paragraph character the run starts at (glyph `index`es count from the paragraph).
    pub start: usize,
    pub font: FontId,
    pub text: String,
    pub glyphs: Vec<Glyph>,
    pub ascent: f64,
    pub descent: f64,
    pub height: f64,
    pub x_advance: f64,
}

/// A laid out line. `x`, `y`, `width` and `height` are its box inside the text's content box.
#[derive(Clone, Debug)]
pub struct Line {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub ascent: f64,
    pub x_advance: f64,
    /// Widths of the whitespace hanging past the left and right edges.
    pub overflow_left: f64,
    pub overflow_right: f64,
    pub runs: Vec<LineRun>,
}

/// The settings of one `Text` that are not per run.
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub max_lines: Option<usize>,
    pub ellipsis: bool,
    pub hyphenation_penalty: Option<f64>,
}

#[cfg(test)]
mod tests;
