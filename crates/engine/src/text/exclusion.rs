//! Exclusions: the shapes of floats that text flows around, and the line boxes left beside them.

/// A float's outline in the text's content box coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Rect { x: f64, y: f64, w: f64, h: f64 },
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64 },
    Polygon(Vec<(f64, f64)>),
}

/// A shape text keeps out of. It also reaches the edge on its float's side (`left` for a left
/// float), so no text lands between a float and the side it is pinned to.
#[derive(Clone, Debug, PartialEq)]
pub struct Exclusion {
    pub shape: Shape,
    pub left: bool,
}

impl Shape {
    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            Shape::Rect { x, y, .. } => (*x, *y) = (*x + dx, *y + dy),
            Shape::Ellipse { cx, cy, .. } => (*cx, *cy) = (*cx + dx, *cy + dy),
            Shape::Polygon(ps) => ps.iter_mut().for_each(|p| *p = (p.0 + dx, p.1 + dy)),
        }
    }

    fn max_y(&self) -> f64 {
        match self {
            Shape::Rect { y, h, .. } => y + h,
            Shape::Ellipse { cy, ry, .. } => cy + ry,
            Shape::Polygon(ps) => ps.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
        }
    }

    /// The x extent the shape covers between `y0` and `y1`. Concave parts of a polygon inside
    /// the band count as covered.
    fn band(&self, y0: f64, y1: f64) -> Option<(f64, f64)> {
        let (x0, x1) = match *self {
            Shape::Rect { x, y, w, h } => {
                if y1 <= y || y0 >= y + h {
                    return None;
                }
                (x, x + w)
            }
            Shape::Ellipse { cx, cy, rx, ry } => {
                if y1 <= cy - ry || y0 >= cy + ry {
                    return None;
                }
                // Widest at the band's y nearest the center.
                let t = if ry == 0.0 {
                    0.0
                } else {
                    (cy.clamp(y0, y1) - cy) / ry
                };
                let half = rx * (1.0 - t * t).sqrt();
                (cx - half, cx + half)
            }
            Shape::Polygon(ref ps) => {
                let mut xs = Vec::new();
                for (i, &(ax, ay)) in ps.iter().enumerate() {
                    let (bx, by) = ps[(i + 1) % ps.len()];
                    if ay >= y0 && ay <= y1 {
                        xs.push(ax);
                    }
                    for y in [y0, y1] {
                        if (ay < y && by > y) || (ay > y && by < y) {
                            xs.push(ax + (y - ay) / (by - ay) * (bx - ax));
                        }
                    }
                }
                let lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                (lo, hi)
            }
        };
        (x1 - x0 > 0.0).then_some((x0, x1))
    }
}

/// A line box: x, y, width and height.
pub type Rect = (f64, f64, f64, f64);

/// The boxes a paragraph's lines go in, from `top` down: bands `band` tall beside the
/// exclusions, each split around them, then the full `width` with no end.
pub fn line_rects(width: f64, top: f64, band: f64, exclusions: &[Exclusion]) -> Vec<Rect> {
    let mut rects = Vec::new();
    let mut y = top;
    if band > 0.0 {
        let bottom = exclusions
            .iter()
            .map(|e| e.shape.max_y())
            .fold(f64::NEG_INFINITY, f64::max);
        while y < bottom {
            let mut parts = vec![(0.0, width)];
            for e in exclusions {
                let Some((x0, x1)) = e.shape.band(y, y + band) else {
                    continue;
                };
                let (x0, x1) = if e.left {
                    (f64::NEG_INFINITY, x1)
                } else {
                    (x0, f64::INFINITY)
                };
                parts = parts
                    .into_iter()
                    .flat_map(|(a, b)| {
                        if x1 <= a || x0 >= b {
                            vec![(a, b)]
                        } else {
                            vec![(a, x0), (x1, b)]
                        }
                    })
                    .filter(|(a, b)| b > a)
                    .collect();
            }
            rects.extend(parts.into_iter().map(|(a, b)| (a, y, b - a, band)));
            y += band;
        }
    }
    rects.push((0.0, y, width, f64::INFINITY));
    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_step_around_a_circle_then_take_the_full_width() {
        let circle = Exclusion {
            shape: Shape::Ellipse {
                cx: 20.0,
                cy: 20.0,
                rx: 20.0,
                ry: 20.0,
            },
            left: true,
        };
        let rects = line_rects(100.0, 0.0, 10.0, &[circle]);
        assert_eq!(rects.len(), 5);
        // The band nearest the middle is the narrowest; the shape's left side is never used.
        assert_eq!(rects[1], (40.0, 10.0, 60.0, 10.0));
        assert!(rects[0].0 > 20.0 && rects[0].0 < 40.0);
        assert_eq!(rects[4], (0.0, 40.0, 100.0, f64::INFINITY));
    }

    #[test]
    fn a_right_triangle_narrows_lines_from_the_right() {
        let tri = Exclusion {
            shape: Shape::Polygon(vec![(100.0, 0.0), (100.0, 20.0), (80.0, 20.0)]),
            left: false,
        };
        let rects = line_rects(100.0, 0.0, 10.0, &[tri]);
        assert_eq!(rects[0], (0.0, 0.0, 90.0, 10.0));
        assert_eq!(rects[1], (0.0, 10.0, 80.0, 10.0));
    }
}
