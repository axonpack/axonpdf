//! Box borders. Each side is stroked along its outer edge at twice the widest width, clipped
//! to the band between the outer and inner rounded boxes and to its own corner joins. Sides
//! are drawn in a turned frame, so one routine draws all four as if it were the top.

use super::Writer;
use super::content::{PageState, path};
use super::emit::Out;
use crate::display::{Border, BorderStyle, Rect, Seg};
use crate::path::{KAPPA, rounded_rect_xy};

impl Writer {
    pub(super) fn draw_border(&mut self, p: &mut PageState, b: &Border, out: Out) {
        let Rect { x, y, w, h } = b.rect;
        let radii = b.radii.map(|r| r.min(w / 2.0).min(h / 2.0).max(0.0));
        // Top, right, bottom, left: a frame whose top edge is that side.
        let frames = [
            ([1.0, 0.0, 0.0, 1.0, x, y], w, h),
            ([0.0, 1.0, -1.0, 0.0, x + w, y], h, w),
            ([-1.0, 0.0, 0.0, -1.0, x + w, y + h], w, h),
            ([0.0, -1.0, 1.0, 0.0, x, y + h], h, w),
        ];
        for (side, (m, fw, fh)) in frames.into_iter().enumerate() {
            let color = b.colors[side];
            let widths = [0, 1, 2, 3].map(|k| b.widths[(side + k) % 4]);
            if widths[0] <= 0.0 || color.a <= 0.0 {
                continue;
            }
            p.save();
            p.transform(m);
            let corners = [0, 1, 2, 3].map(|k| radii[(side + k) % 4]);
            clip_band(p, fw, fh, widths, corners);
            p.c.set_stroke_rgb(color.r, color.g, color.b);
            self.set_alpha(p, 1.0, color.a, out);
            let [wt, wr, _, wl] = widths;
            p.c.set_line_width(wl.max(wt).max(wr) * 2.0);
            match b.styles[side] {
                BorderStyle::Solid => {}
                BorderStyle::Dashed => {
                    p.c.set_dash_pattern([wt * 2.0, wt * 1.2], 0.0);
                }
                BorderStyle::Dotted => {
                    p.c.set_dash_pattern([wt, wt * 1.2], 0.0);
                }
            }
            let [tl, tr, ..] = corners;
            let k = 1.0 - KAPPA;
            let edge = [
                Seg::Move(0.0, tl.max(wt)),
                Seg::Cubic(0.0, tl * k, tl * k, 0.0, tl, 0.0),
                Seg::Line(fw - tr, 0.0),
                Seg::Cubic(fw - tr * k, 0.0, fw, tr * k, fw, tr),
            ];
            path(&mut p.c, &edge);
            p.c.stroke();
            p.restore();
        }
    }
}

/// Clips to the top band of a `fw` × `fh` frame: inside the outer box, outside the inner one,
/// and between the two diagonal joins with the neighbouring sides.
fn clip_band(p: &mut PageState, fw: f32, fh: f32, widths: [f32; 4], radii: [f32; 4]) {
    let [wt, wr, wb, wl] = widths;
    let [tl, tr, br, bl] = radii;
    let mut band = rounded_rect_xy(
        Rect {
            x: 0.0,
            y: 0.0,
            w: fw,
            h: fh,
        },
        radii.map(|r| (r, r)),
    );
    let inner = Rect {
        x: wl,
        y: wt,
        w: fw - wl - wr,
        h: fh - wt - wb,
    };
    if inner.w > 0.0 && inner.h > 0.0 {
        let d = |r: f32, by: f32| (r - by).max(0.0);
        let inner_radii = [
            (d(tl, wl), d(tl, wt)),
            (d(tr, wr), d(tr, wt)),
            (d(br, wr), d(br, wb)),
            (d(bl, wl), d(bl, wb)),
        ];
        band.extend(rounded_rect_xy(inner, inner_radii));
    }
    path(&mut p.c, &band);
    p.c.clip_even_odd();
    p.c.end_path();
    // The joins run from each outer corner through the inner one, up to where they would meet.
    let reach = if wl + wr > 0.0 {
        fw / (wl + wr)
    } else {
        f32::INFINITY
    };
    let t = (fh / (2.0 * wt)).min(reach);
    let wedge = [
        Seg::Move(0.0, 0.0),
        Seg::Line(fw, 0.0),
        Seg::Line(fw - wr * t, wt * t),
        Seg::Line(wl * t, wt * t),
        Seg::Close,
    ];
    path(&mut p.c, &wedge);
    p.c.clip_nonzero();
    p.c.end_path();
}
