//! Gradient paints.

use super::Writer;
use super::content::PageState;
use super::emit::Out;
use crate::display::Gradient;

impl Writer {
    /// Sets a gradient as the fill (or stroke) paint. Returns the alpha to draw with.
    pub(super) fn set_gradient(
        &mut self,
        p: &mut PageState,
        g: &Gradient,
        fill: bool,
        _out: Out,
    ) -> f32 {
        let c = g
            .stops
            .first()
            .map_or(crate::display::Color::BLACK, |s| s.1);
        if fill {
            p.c.set_fill_rgb(c.r, c.g, c.b)
        } else {
            p.c.set_stroke_rgb(c.r, c.g, c.b)
        };
        c.a
    }
}
