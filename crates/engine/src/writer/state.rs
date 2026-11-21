//! The graphics state the writer tracks while it turns ops into a content stream.

use pdf_writer::{Content, Ref};

use super::Resources;
use crate::display::{Rect, Seg};
use crate::path::{apply, multiply};

/// The parts of the graphics state the writer tracks.
#[derive(Clone, Copy)]
pub(super) struct Gfx {
    /// From top-left points to PDF space.
    pub ctm: [f32; 6],
    /// Multiplied into every alpha.
    pub opacity: f32,
    /// The (fill, stroke) alpha currently set in the PDF.
    pub alpha: (f32, f32),
}

pub(super) struct PageState {
    pub c: Content,
    pub res: Resources,
    pub gfx: Gfx,
    stack: Vec<Gfx>,
    pub annots: Vec<Ref>,
}

impl PageState {
    pub fn new(gfx: Gfx) -> Self {
        PageState {
            c: Content::new(),
            res: Resources::default(),
            gfx,
            stack: Vec::new(),
            annots: Vec::new(),
        }
    }

    pub fn save(&mut self) {
        self.c.save_state();
        self.stack.push(self.gfx);
    }

    pub fn restore(&mut self) {
        if let Some(g) = self.stack.pop() {
            self.c.restore_state();
            self.gfx = g;
        }
    }

    pub fn transform(&mut self, m: [f32; 6]) {
        self.c.transform(m);
        self.gfx.ctm = multiply(self.gfx.ctm, m);
    }

    /// The bounding box of a rect after the current transform, as `[x0 y0 x1 y1]`.
    pub fn to_pdf(&self, r: Rect) -> [f32; 4] {
        let pts = [
            (r.x, r.y),
            (r.x + r.w, r.y),
            (r.x, r.y + r.h),
            (r.x + r.w, r.y + r.h),
        ];
        let pts = pts.map(|(x, y)| apply(self.gfx.ctm, x, y));
        let xs = pts.map(|p| p.0);
        let ys = pts.map(|p| p.1);
        let min = |v: [f32; 4]| v.into_iter().fold(f32::INFINITY, f32::min);
        let max = |v: [f32; 4]| v.into_iter().fold(f32::NEG_INFINITY, f32::max);
        [min(xs), min(ys), max(xs), max(ys)]
    }
}

/// Emits a path. Segments with a NaN or infinite coordinate are left out: one such number
/// would end the page's drawing in viewers.
pub(super) fn path(c: &mut Content, segs: &[Seg]) {
    for s in segs {
        let finite = match *s {
            Seg::Move(x, y) | Seg::Line(x, y) => x.is_finite() && y.is_finite(),
            Seg::Cubic(a, b, c2, d, x, y) => [a, b, c2, d, x, y].iter().all(|v| v.is_finite()),
            Seg::Close => true,
        };
        if !finite {
            continue;
        }
        match *s {
            Seg::Move(x, y) => c.move_to(x, y),
            Seg::Line(x, y) => c.line_to(x, y),
            Seg::Cubic(x1, y1, x2, y2, x, y) => c.cubic_to(x1, y1, x2, y2, x, y),
            Seg::Close => c.close_path(),
        };
    }
}
