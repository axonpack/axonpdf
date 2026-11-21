//! Builds paths the way a canvas does: it remembers the current point, so quadratic curves and
//! SVG arcs can be turned into cubics.

use std::f32::consts::{FRAC_PI_2, PI};

use crate::display::Seg;

#[derive(Default)]
pub struct PathBuilder {
    pub segs: Vec<Seg>,
    /// Current point and the start of the open subpath.
    pos: (f32, f32),
    start: (f32, f32),
}

impl PathBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pos(&self) -> (f32, f32) {
        self.pos
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.segs.push(Seg::Move(x, y));
        self.pos = (x, y);
        self.start = (x, y);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.segs.push(Seg::Line(x, y));
        self.pos = (x, y);
    }

    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.segs.push(Seg::Cubic(x1, y1, x2, y2, x, y));
        self.pos = (x, y);
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (x0, y0) = self.pos;
        let t = 2.0 / 3.0;
        self.cubic_to(
            x0 + t * (cx - x0),
            y0 + t * (cy - y0),
            x + t * (cx - x),
            y + t * (cy - y),
            x,
            y,
        );
    }

    pub fn close(&mut self) {
        self.segs.push(Seg::Close);
        self.pos = self.start;
    }

    /// An SVG elliptical arc from the current point to `(x, y)`, as cubics of at most 90°.
    #[allow(clippy::too_many_arguments)]
    pub fn arc_to(
        &mut self,
        rx: f32,
        ry: f32,
        rotation_deg: f32,
        large: bool,
        sweep: bool,
        x: f32,
        y: f32,
    ) {
        let (x1, y1) = self.pos;
        let (mut rx, mut ry) = (rx.abs(), ry.abs());
        if (x1 == x && y1 == y) || rx == 0.0 || ry == 0.0 {
            if (x1, y1) != (x, y) {
                self.line_to(x, y);
            }
            return;
        }
        let phi = rotation_deg.to_radians();
        let (sin, cos) = phi.sin_cos();
        // Endpoint to center parameterization (SVG 1.1, appendix F.6.5).
        let (dx, dy) = ((x1 - x) / 2.0, (y1 - y) / 2.0);
        let (px, py) = (cos * dx + sin * dy, -sin * dx + cos * dy);
        let lambda = (px * px) / (rx * rx) + (py * py) / (ry * ry);
        if lambda > 1.0 {
            rx *= lambda.sqrt();
            ry *= lambda.sqrt();
        }
        let num = rx * rx * ry * ry - rx * rx * py * py - ry * ry * px * px;
        let den = rx * rx * py * py + ry * ry * px * px;
        let mut coef = (num / den).max(0.0).sqrt();
        if large == sweep {
            coef = -coef;
        }
        let (cxp, cyp) = (coef * rx * py / ry, -coef * ry * px / rx);
        let cx = cos * cxp - sin * cyp + (x1 + x) / 2.0;
        let cy = sin * cxp + cos * cyp + (y1 + y) / 2.0;
        let angle = |ux: f32, uy: f32| uy.atan2(ux);
        let theta = angle((px - cxp) / rx, (py - cyp) / ry);
        let mut delta = angle((-px - cxp) / rx, (-py - cyp) / ry) - theta;
        if sweep && delta < 0.0 {
            delta += 2.0 * PI;
        } else if !sweep && delta > 0.0 {
            delta -= 2.0 * PI;
        }
        // The small margin keeps exact quarter turns from rounding up to one more segment.
        let n = (delta.abs() / FRAC_PI_2 - 1e-4).ceil().max(1.0) as usize;
        let step = delta / n as f32;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let point = |t: f32| {
            let (s, c) = t.sin_cos();
            (
                cx + rx * c * cos - ry * s * sin,
                cy + rx * c * sin + ry * s * cos,
            )
        };
        let deriv = |t: f32| {
            let (s, c) = t.sin_cos();
            (-rx * s * cos - ry * c * sin, -rx * s * sin + ry * c * cos)
        };
        for i in 0..n {
            let (t0, t1) = (theta + step * i as f32, theta + step * (i + 1) as f32);
            let (p0, p1) = (point(t0), point(t1));
            let (d0, d1) = (deriv(t0), deriv(t1));
            let end = if i + 1 == n { (x, y) } else { p1 };
            self.cubic_to(
                p0.0 + k * d0.0,
                p0.1 + k * d0.1,
                p1.0 - k * d1.0,
                p1.1 - k * d1.1,
                end.0,
                end.1,
            );
        }
    }
}
