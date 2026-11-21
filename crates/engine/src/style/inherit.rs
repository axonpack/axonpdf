//! Inheritance: text properties flow from a node to its children.

use super::Style;

/// Fills `child` with what `parent` passes down. Every node passes color, font, letter
/// spacing, opacity, text decoration (joined with the child's own), transform, line height,
/// alignment, visibility and word spacing; a text node also passes its background color to the
/// text nested in it.
pub fn inherit(parent: &Style, parent_is_text: bool, child: &mut Style) {
    macro_rules! pass {
        ($($f:ident),*) => { $( if child.$f.is_none() { child.$f = parent.$f.clone(); } )* };
    }
    pass!(
        color,
        font_family,
        font_size,
        font_style,
        font_weight,
        letter_spacing,
        opacity,
        text_transform,
        line_height,
        text_align,
        visibility,
        word_spacing
    );
    if parent_is_text {
        pass!(background_color);
    }
    let joined: Vec<&str> = [
        parent.text_decoration.as_deref(),
        child.text_decoration.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|d| !d.is_empty() && *d != "none")
    .collect();
    if parent.text_decoration.is_some() {
        child.text_decoration = Some(joined.join(" "));
    }
}
