//! A page's display list as a content stream, plus the resources and annotations it uses.

use pdf_writer::{Name, Ref};

use super::emit::Out;
use super::state::Gfx;
pub(super) use super::state::{PageState, path};
use super::{Resources, Writer};
use crate::display::{Color, Fill, FillRule, LineCap, LineJoin, Op, PageOut, Paint, Seg, Stroke};
use crate::fonts::FontStore;
use crate::images::{ImageStore, placement};
use crate::path::{apply, rounded_rect};

pub(super) struct PageParts {
    pub content: Vec<u8>,
    pub resources: Resources,
    pub annots: Vec<Ref>,
}

impl Writer {
    pub(super) fn content(
        &mut self,
        page: &PageOut,
        page_ref: Ref,
        fonts: &FontStore,
        images: &mut ImageStore,
        out: Out,
    ) -> PageParts {
        let flip = [1.0, 0.0, 0.0, -1.0, 0.0, page.height];
        let gfx = Gfx {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            opacity: 1.0,
            alpha: (1.0, 1.0),
        };
        let mut p = PageState::new(gfx);
        p.transform(flip);
        for op in &page.ops {
            match op {
                Op::Save => p.save(),
                Op::Restore => p.restore(),
                Op::Transform(m) => p.transform(*m),
                Op::Opacity(o) => p.gfx.opacity *= o,
                Op::Clip { path: segs, rule } => {
                    path(&mut p.c, segs);
                    match rule {
                        FillRule::NonZero => p.c.clip_nonzero(),
                        FillRule::EvenOdd => p.c.clip_even_odd(),
                    };
                    p.c.end_path();
                }
                Op::Rect {
                    rect,
                    radii,
                    fill,
                    stroke,
                } => {
                    let fill = fill.clone().map(|paint| Fill {
                        paint,
                        rule: FillRule::NonZero,
                    });
                    self.draw_path(
                        &mut p,
                        &rounded_rect(*rect, *radii),
                        fill.as_ref(),
                        stroke.as_ref(),
                        out,
                    );
                }
                Op::Path {
                    path: segs,
                    fill,
                    stroke,
                } => self.draw_path(&mut p, segs, fill.as_ref(), stroke.as_ref(), out),
                Op::Border(b) => self.draw_border(&mut p, b, out),
                Op::Text(run) => {
                    let alpha = self.set_fill_paint(&mut p, &Paint::Solid(run.color), out);
                    self.set_alpha(&mut p, alpha, 1.0, out);
                    let name = self.draw_text(&mut p.c, run, fonts);
                    let (_, r) = self.font_ref(run.font);
                    Resources::add(&mut p.res.fonts, name, r);
                }
                Op::Image { id, rect, opacity } => {
                    let (name, r) = self.image_ref(*id, images, out);
                    Resources::add(&mut p.res.x_objects, name.clone(), r);
                    p.save();
                    self.set_alpha(&mut p, *opacity, 1.0, out);
                    p.transform(placement(
                        images.get(*id).orientation,
                        rect.x,
                        rect.y,
                        rect.w,
                        rect.h,
                    ));
                    p.c.x_object(Name(name.as_bytes()));
                    p.restore();
                }
                Op::Link { rect, target } => {
                    let a = self.link(p.to_pdf(*rect), target, out);
                    p.annots.push(a);
                }
                Op::Dest { name, x, y } => self.add_dest(name, page_ref, *x, *y, page.height),
                Op::Bookmark(b) => self.add_bookmark(b, page_ref, page.width, page.height),
                Op::Note { x, y, text, color } => {
                    let (px, py) = apply(p.gfx.ctm, *x, *y);
                    let a = self.note((px, py), text, *color, out);
                    p.annots.push(a);
                }
                Op::FieldSet { id, name, parent } => self.add_field_set(*id, name, *parent),
                Op::Field(f) => {
                    let a = self.field(f, p.to_pdf(f.rect), out);
                    p.annots.push(a);
                }
            }
        }
        PageParts {
            content: p.c.finish().to_vec(),
            resources: p.res,
            annots: p.annots,
        }
    }

    /// Sets the ExtGState for these alphas (before the page opacity) if they changed.
    pub(super) fn set_alpha(&mut self, p: &mut PageState, fill: f32, stroke: f32, out: Out) {
        let want = (fill * p.gfx.opacity, stroke * p.gfx.opacity);
        if want != p.gfx.alpha {
            let (name, r) = self.alpha_state(want.0, want.1, out);
            Resources::add(&mut p.res.ext_g_states, name.clone(), r);
            p.c.set_parameters(Name(name.as_bytes()));
            p.gfx.alpha = want;
        }
    }

    /// Sets the fill color or pattern. Returns the paint's own alpha.
    pub(super) fn set_fill_paint(&mut self, p: &mut PageState, paint: &Paint, out: Out) -> f32 {
        match paint {
            Paint::Solid(Color { r, g, b, a }) => {
                p.c.set_fill_rgb(*r, *g, *b);
                *a
            }
            Paint::Gradient(g) => self.set_gradient(p, g, true, out),
        }
    }

    pub(super) fn set_stroke(&mut self, p: &mut PageState, s: &Stroke, out: Out) -> f32 {
        let alpha = match &s.paint {
            Paint::Solid(Color { r, g, b, a }) => {
                p.c.set_stroke_rgb(*r, *g, *b);
                *a
            }
            Paint::Gradient(g) => self.set_gradient(p, g, false, out),
        };
        p.c.set_line_width(s.width);
        p.c.set_line_cap(match s.cap {
            LineCap::Butt => pdf_writer::types::LineCapStyle::ButtCap,
            LineCap::Round => pdf_writer::types::LineCapStyle::RoundCap,
            LineCap::Square => pdf_writer::types::LineCapStyle::ProjectingSquareCap,
        });
        p.c.set_line_join(match s.join {
            LineJoin::Miter => pdf_writer::types::LineJoinStyle::MiterJoin,
            LineJoin::Round => pdf_writer::types::LineJoinStyle::RoundJoin,
            LineJoin::Bevel => pdf_writer::types::LineJoinStyle::BevelJoin,
        });
        p.c.set_miter_limit(s.miter_limit);
        p.c.set_dash_pattern(s.dash.iter().copied(), s.dash_phase);
        alpha
    }

    pub(super) fn draw_path(
        &mut self,
        p: &mut PageState,
        segs: &[Seg],
        fill: Option<&Fill>,
        stroke: Option<&Stroke>,
        out: Out,
    ) {
        if fill.is_none() && stroke.is_none() {
            return;
        }
        let masked = [fill.map(|f| &f.paint), stroke.map(|s| &s.paint)]
            .into_iter()
            .any(
                |paint| matches!(paint, Some(Paint::Gradient(g)) if super::gradient::has_alpha(g)),
            );
        if masked {
            // The soft mask stays in the graphics state until restored.
            p.save();
        }
        let fa = fill.map_or(1.0, |f| self.set_fill_paint(p, &f.paint, out));
        let sa = stroke.map_or(1.0, |s| self.set_stroke(p, s, out));
        self.set_alpha(p, fa, sa, out);
        path(&mut p.c, segs);
        let even_odd = fill.is_some_and(|f| f.rule == FillRule::EvenOdd);
        match (fill.is_some(), stroke.is_some(), even_odd) {
            (true, true, false) => p.c.fill_nonzero_and_stroke(),
            (true, true, true) => p.c.fill_even_odd_and_stroke(),
            (true, false, false) => p.c.fill_nonzero(),
            (true, false, true) => p.c.fill_even_odd(),
            _ => p.c.stroke(),
        };
        if masked {
            p.restore();
        }
    }
}
