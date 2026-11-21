//! Linear and radial gradients as shading patterns. Stops with alpha add a luminosity soft
//! mask drawn from the same gradient in gray.

use std::collections::HashMap;

use pdf_writer::types::{ColorSpaceOperand, FunctionShadingType, MaskType};
use pdf_writer::{Content, Finish, Name, Rect, Ref};

use super::emit::Out;
use super::state::PageState;
use super::{Resources, Writer};
use crate::display::{Color, Gradient, GradientKind};
use crate::path::multiply;

/// Far enough to cover any page in any user space the mask is drawn in.
const HUGE: f32 = 1.0e5;

impl Writer {
    /// Sets a gradient as the fill (or stroke) paint. Returns the alpha left to apply (1; stop
    /// alpha goes through the soft mask, which needs the caller's save and restore).
    pub(super) fn set_gradient(
        &mut self,
        p: &mut PageState,
        g: &Gradient,
        fill: bool,
        out: Out,
    ) -> f32 {
        let pattern = self.pattern(
            g,
            multiply(p.gfx.ctm, g.transform),
            |c| vec![c.r, c.g, c.b],
            out,
        );
        let name = format!("P{}", pattern.get());
        Resources::add(&mut p.res.patterns, name.clone(), pattern);
        if fill {
            p.c.set_fill_color_space(ColorSpaceOperand::Pattern);
            p.c.set_fill_pattern(None, Name(name.as_bytes()));
        } else {
            p.c.set_stroke_color_space(ColorSpaceOperand::Pattern);
            p.c.set_stroke_pattern(None, Name(name.as_bytes()));
        }
        if has_alpha(g) {
            self.soft_mask(p, g, out);
        }
        1.0
    }

    fn soft_mask(&mut self, p: &mut PageState, g: &Gradient, out: Out) {
        let key = mask_key(g);
        let state = match self.gradients.masks.get(&key) {
            Some(r) => *r,
            None => {
                let r = self.write_mask(g, out);
                self.gradients.masks.insert(key, r);
                r
            }
        };
        let name = format!("Gs{}", state.get());
        Resources::add(&mut p.res.ext_g_states, name.clone(), state);
        p.c.set_parameters(Name(name.as_bytes()));
    }

    /// The ExtGState holding a gradient's alpha as a luminosity mask. It does not depend on the
    /// page, so each gradient's mask is written once.
    fn write_mask(&mut self, g: &Gradient, out: Out) -> Ref {
        let pattern = self.pattern(g, g.transform, |c| vec![c.a], out);
        let mut content = Content::new();
        content.set_fill_color_space(ColorSpaceOperand::Pattern);
        content.set_fill_pattern(None, Name(b"M"));
        content.rect(-HUGE, -HUGE, 2.0 * HUGE, 2.0 * HUGE);
        content.fill_nonzero();
        let form = self.emit.alloc();
        let data = self.seal(form, content.finish().to_vec());
        self.emit.write(out, |c| {
            let mut x = c.form_xobject(form, &data);
            x.bbox(Rect::new(-HUGE, -HUGE, HUGE, HUGE));
            x.group().transparency().color_space().device_gray();
            x.resources().patterns().pair(Name(b"M"), pattern);
        });
        let state = self.emit.alloc();
        self.emit.write(out, |c| {
            c.ext_graphics(state)
                .soft_mask()
                .subtype(MaskType::Luminosity)
                .group(form);
        });
        state
    }

    /// A shading pattern whose colors are `channels` of each stop (RGB, or alpha as gray).
    fn pattern(
        &mut self,
        g: &Gradient,
        matrix: [f32; 6],
        channels: impl Fn(&Color) -> Vec<f32>,
        out: Out,
    ) -> Ref {
        let function = self.stops_function(g, &channels, out);
        let gray = channels(&Color::BLACK).len() == 1;
        let r = self.emit.alloc();
        self.emit.write(out, |c| {
            let mut pat = c.shading_pattern(r);
            pat.matrix(matrix);
            let mut s = pat.function_shading();
            let space = s.color_space();
            if gray {
                space.device_gray();
            } else {
                space.device_rgb();
            }
            match g.kind {
                GradientKind::Linear { x1, y1, x2, y2 } => {
                    s.shading_type(FunctionShadingType::Axial)
                        .coords([x1, y1, x2, y2]);
                }
                GradientKind::Radial {
                    fx,
                    fy,
                    fr,
                    cx,
                    cy,
                    r,
                } => {
                    s.shading_type(FunctionShadingType::Radial)
                        .coords([fx, fy, fr, cx, cy, r]);
                }
            }
            s.extend([true, true]).function(function);
            s.finish();
        });
        r
    }

    /// One exponential function per pair of stops, stitched when there are more than two.
    fn stops_function(
        &mut self,
        g: &Gradient,
        channels: &impl Fn(&Color) -> Vec<f32>,
        out: Out,
    ) -> Ref {
        let key: Vec<u32> = g
            .stops
            .iter()
            .flat_map(|(o, c)| std::iter::once(*o).chain(channels(c)))
            .map(f32::to_bits)
            .collect();
        if let Some(r) = self.gradients.functions.get(&key) {
            return *r;
        }
        let r = self.write_function(g, channels, out);
        self.gradients.functions.insert(key, r);
        r
    }

    fn write_function(
        &mut self,
        g: &Gradient,
        channels: &impl Fn(&Color) -> Vec<f32>,
        out: Out,
    ) -> Ref {
        let mut stops = g.stops.clone();
        if stops.is_empty() {
            stops.push((0.0, Color::BLACK));
        }
        if stops[0].0 > 0.0 {
            stops.insert(0, (0.0, stops[0].1));
        }
        if stops[stops.len() - 1].0 < 1.0 || stops.len() == 1 {
            stops.push((1.0, stops[stops.len() - 1].1));
        }
        let segments: Vec<Ref> = stops
            .windows(2)
            .map(|pair| {
                let r = self.emit.alloc();
                let (c0, c1) = (channels(&pair[0].1), channels(&pair[1].1));
                self.emit.write(out, |c| {
                    c.exponential_function(r)
                        .domain([0.0, 1.0])
                        .c0(c0)
                        .c1(c1)
                        .n(1.0);
                });
                r
            })
            .collect();
        if segments.len() == 1 {
            return segments[0];
        }
        let r = self.emit.alloc();
        let bounds: Vec<f32> = stops[1..stops.len() - 1]
            .iter()
            .map(|s| s.0.clamp(0.0, 1.0))
            .collect();
        self.emit.write(out, |c| {
            let mut f = c.stitching_function(r);
            f.domain([0.0, 1.0])
                .functions(segments.iter().copied())
                .bounds(bounds);
            f.encode((0..segments.len()).flat_map(|_| [0.0, 1.0]));
        });
        r
    }
}

/// Functions and masks already written, by what they draw.
#[derive(Default)]
pub(super) struct Written {
    functions: HashMap<Vec<u32>, Ref>,
    masks: HashMap<Vec<u32>, Ref>,
}

fn mask_key(g: &Gradient) -> Vec<u32> {
    let coords: Vec<f32> = match g.kind {
        GradientKind::Linear { x1, y1, x2, y2 } => vec![x1, y1, x2, y2],
        GradientKind::Radial {
            fx,
            fy,
            fr,
            cx,
            cy,
            r,
        } => vec![fx, fy, fr, cx, cy, r, f32::NAN],
    };
    let stops = g.stops.iter().flat_map(|(o, c)| [*o, c.a]);
    coords
        .into_iter()
        .chain(g.transform)
        .chain(stops)
        .map(f32::to_bits)
        .collect()
}

pub(super) fn has_alpha(g: &Gradient) -> bool {
    g.stops.iter().any(|s| s.1.a < 1.0)
}
