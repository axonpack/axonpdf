//! Styles: raw styles as written, resolved into one typed [`Style`] per node.
//!
//! The steps follow the components' own order: flatten arrays, apply media queries against the
//! page, then expand each key in the order it was written (so `margin` after `marginTop`
//! overrides it, and the other way round), then inherit text properties from the parent.

pub mod color;
mod expand;
pub mod flatten;
mod inherit;
mod shorthand;
pub mod units;

pub use inherit::inherit;
pub use units::{Container, Dim};

use crate::display::{BorderStyle, Color};
use crate::layout::yoga::{Align, FlexDirection, Justify, Overflow, PositionType, Wrap};
use serde_json::Value;

/// Edges are top, right, bottom, left. Corners are top left, top right, bottom right, bottom left.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    // Box and flex.
    pub display_none: bool,
    pub position: Option<PositionType>,
    pub inset: [Option<Dim>; 4],
    pub width: Option<Dim>,
    pub height: Option<Dim>,
    pub min_width: Option<Dim>,
    pub min_height: Option<Dim>,
    pub max_width: Option<Dim>,
    pub max_height: Option<Dim>,
    pub margin: [Option<Dim>; 4],
    pub padding: [Option<Dim>; 4],
    pub border_width: [Option<f64>; 4],
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<Wrap>,
    pub flex_grow: Option<f64>,
    pub flex_shrink: Option<f64>,
    pub flex_basis: Option<Dim>,
    pub align_items: Option<Align>,
    pub align_self: Option<Align>,
    pub align_content: Option<Align>,
    pub justify_content: Option<Justify>,
    pub row_gap: Option<Dim>,
    pub column_gap: Option<Dim>,
    pub aspect_ratio: Option<f64>,
    pub overflow: Option<Overflow>,
    pub z_index: Option<f64>,
    pub float: Option<String>,

    // Paint.
    pub background_color: Option<Color>,
    pub opacity: Option<f64>,
    pub border_color: [Option<Color>; 4],
    pub border_style: [Option<BorderStyle>; 4],
    pub border_radius: [Option<Dim>; 4],
    pub transform: Option<Value>,
    pub transform_origin: Option<Value>,
    pub object_fit: Option<String>,
    pub object_position: [Option<Dim>; 2],

    // Text.
    pub color: Option<Color>,
    pub font_family: Option<Vec<String>>,
    pub font_size: Option<f64>,
    pub font_style: Option<String>,
    pub font_weight: Option<u16>,
    pub letter_spacing: Option<f64>,
    pub line_height: Option<f64>,
    pub text_align: Option<String>,
    pub text_decoration: Option<String>,
    pub text_decoration_color: Option<Color>,
    pub text_decoration_style: Option<String>,
    pub text_transform: Option<String>,
    pub text_indent: Option<f64>,
    pub max_lines: Option<f64>,
    pub text_overflow: Option<String>,
    pub vertical_align: Option<String>,
    pub direction: Option<String>,
    pub visibility: Option<String>,
    pub word_spacing: Option<f64>,
}

/// Resolves a raw style (object, array, media queries) against the page.
pub fn resolve(c: &Container, raw: &Value) -> Style {
    let flat = flatten::media(c, flatten::flatten(raw));
    let mut style = Style::default();
    for (key, value) in &flat {
        expand::apply(&mut style, c, key, value, &flat);
    }
    // Floats leave the flow.
    if matches!(style.float.as_deref(), Some("left" | "right")) {
        style.position = Some(PositionType::Absolute);
    }
    style
}

pub const TOP: usize = 0;
pub const RIGHT: usize = 1;
pub const BOTTOM: usize = 2;
pub const LEFT: usize = 3;

#[cfg(test)]
mod tests;
