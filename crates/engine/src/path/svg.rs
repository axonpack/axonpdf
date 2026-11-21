//! SVG path data (`d`), point lists, shapes and marker positions.

use super::builder::PathBuilder;
use crate::display::Seg;

/// Parses path data. Parsing stops at the first error and keeps what came before, as SVG
/// renderers do.
pub fn parse(d: &str) -> Vec<Seg> {
    let mut p = PathBuilder::new();
    let mut tokens = Tokens {
        s: d.as_bytes(),
        i: 0,
    };
    let mut cmd = 0u8;
    // The last cubic or quadratic control point, for S and T.
    let mut last_ctrl: Option<(u8, f32, f32)> = None;
    while let Some(next) = tokens.command_or_number() {
        if let Some(c) = next {
            cmd = c;
        } else if cmd == 0 {
            break;
        }
        let rel = cmd.is_ascii_lowercase();
        let (ox, oy) = if rel { p.pos() } else { (0.0, 0.0) };
        let upper = cmd.to_ascii_uppercase();
        let ok = (|| {
            match upper {
                b'M' => {
                    let (x, y) = (tokens.num()? + ox, tokens.num()? + oy);
                    p.move_to(x, y);
                    // Further pairs are line-tos.
                    cmd = if rel { b'l' } else { b'L' };
                }
                b'L' => {
                    let (x, y) = (tokens.num()? + ox, tokens.num()? + oy);
                    p.line_to(x, y);
                }
                b'H' => p.line_to(tokens.num()? + ox, p.pos().1),
                b'V' => p.line_to(p.pos().0, tokens.num()? + oy),
                b'C' | b'S' => {
                    let (x1, y1) = if upper == b'C' {
                        (tokens.num()? + ox, tokens.num()? + oy)
                    } else {
                        reflect(last_ctrl, b'C', p.pos())
                    };
                    let (x2, y2, x, y) = (
                        tokens.num()? + ox,
                        tokens.num()? + oy,
                        tokens.num()? + ox,
                        tokens.num()? + oy,
                    );
                    p.cubic_to(x1, y1, x2, y2, x, y);
                    last_ctrl = Some((b'C', x2, y2));
                    return Some(());
                }
                b'Q' | b'T' => {
                    let (cx, cy) = if upper == b'Q' {
                        (tokens.num()? + ox, tokens.num()? + oy)
                    } else {
                        reflect(last_ctrl, b'Q', p.pos())
                    };
                    let (x, y) = (tokens.num()? + ox, tokens.num()? + oy);
                    p.quad_to(cx, cy, x, y);
                    last_ctrl = Some((b'Q', cx, cy));
                    return Some(());
                }
                b'A' => {
                    let (rx, ry, rot) = (tokens.num()?, tokens.num()?, tokens.num()?);
                    let (large, sweep) = (tokens.flag()?, tokens.flag()?);
                    let (x, y) = (tokens.num()? + ox, tokens.num()? + oy);
                    p.arc_to(rx, ry, rot, large, sweep, x, y);
                }
                b'Z' => p.close(),
                _ => return None,
            }
            last_ctrl = None;
            Some(())
        })();
        if ok.is_none() {
            break;
        }
        if upper == b'Z' {
            cmd = 0;
        }
    }
    p.segs
}

/// The control point S or T starts with: the previous one mirrored, if the previous command
/// was of the same kind.
fn reflect(last: Option<(u8, f32, f32)>, kind: u8, (x, y): (f32, f32)) -> (f32, f32) {
    match last {
        Some((k, cx, cy)) if k == kind => (2.0 * x - cx, 2.0 * y - cy),
        _ => (x, y),
    }
}

/// `points` attribute values: `"x1,y1 x2,y2 ..."`. An odd number drops the last.
pub fn points(s: &str) -> Vec<(f32, f32)> {
    let mut t = Tokens {
        s: s.as_bytes(),
        i: 0,
    };
    std::iter::from_fn(|| Some((t.num()?, t.num()?))).collect()
}

/// A polyline, or a polygon when `closed`.
pub fn polyline(pts: &[(f32, f32)], closed: bool) -> Vec<Seg> {
    let mut segs: Vec<Seg> = pts
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            if i == 0 {
                Seg::Move(x, y)
            } else {
                Seg::Line(x, y)
            }
        })
        .collect();
    if closed && !segs.is_empty() {
        segs.push(Seg::Close);
    }
    segs
}

/// SVG `<rect>`: square corners unless both `rx` and `ry` are set.
pub fn rect(x: f32, y: f32, w: f32, h: f32, rx: f32, ry: f32) -> Vec<Seg> {
    if w == 0.0 || h == 0.0 {
        return Vec::new();
    }
    if rx != 0.0 && ry != 0.0 {
        return super::rounded_rect_xy(crate::display::Rect { x, y, w, h }, [(rx, ry); 4]);
    }
    vec![
        Seg::Move(x, y),
        Seg::Line(x + w, y),
        Seg::Line(x + w, y + h),
        Seg::Line(x, y + h),
        Seg::Close,
    ]
}

/// Where markers go: every vertex with the direction a marker there points, the bisector of
/// the incoming and outgoing directions at middle vertices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    /// Radians, clockwise from +x in top-left space.
    pub angle: f32,
}

/// Marker vertices: start, the middle ones, end.
pub fn vertices(segs: &[Seg]) -> Vec<Vertex> {
    // Each vertex with the direction coming in and going out.
    let mut pts: Vec<(f32, f32, Option<f32>, Option<f32>)> = Vec::new();
    let mut start = (0.0, 0.0);
    let dir = |a: (f32, f32), b: (f32, f32)| (a != b).then(|| (b.1 - a.1).atan2(b.0 - a.0));
    for s in segs {
        let from = pts.last().map(|p| (p.0, p.1)).unwrap_or_default();
        let (to, d_in, d_out) = match *s {
            Seg::Move(x, y) => {
                start = (x, y);
                pts.push((x, y, None, None));
                continue;
            }
            Seg::Line(x, y) => ((x, y), dir(from, (x, y)), dir(from, (x, y))),
            Seg::Cubic(x1, y1, x2, y2, x, y) => {
                let first = dir(from, (x1, y1))
                    .or(dir(from, (x2, y2)))
                    .or(dir(from, (x, y)));
                let last = dir((x2, y2), (x, y))
                    .or(dir((x1, y1), (x, y)))
                    .or(dir(from, (x, y)));
                ((x, y), last, first)
            }
            Seg::Close => (start, dir(from, start), dir(from, start)),
        };
        if let Some(prev) = pts.last_mut() {
            prev.3 = prev.3.or(d_out);
        }
        pts.push((to.0, to.1, d_in, None));
    }
    let n = pts.len();
    pts.iter()
        .enumerate()
        .map(|(i, &(x, y, d_in, d_out))| {
            let angle = match (d_in, d_out) {
                (Some(a), Some(b)) if i > 0 && i + 1 < n => bisect(a, b),
                (Some(a), _) => a,
                (None, Some(b)) => b,
                (None, None) => 0.0,
            };
            Vertex { x, y, angle }
        })
        .collect()
}

fn bisect(a: f32, b: f32) -> f32 {
    use std::f32::consts::PI;
    let mut diff = b - a;
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }
    a + diff / 2.0
}

struct Tokens<'a> {
    s: &'a [u8],
    i: usize,
}

impl Tokens<'_> {
    fn skip(&mut self) {
        while self.i < self.s.len()
            && (self.s[self.i].is_ascii_whitespace() || self.s[self.i] == b',')
        {
            self.i += 1;
        }
    }

    /// `Some(Some(cmd))` for a command letter, `Some(None)` when a number follows (an implicit
    /// repeat), `None` at the end.
    fn command_or_number(&mut self) -> Option<Option<u8>> {
        self.skip();
        let c = *self.s.get(self.i)?;
        if c.is_ascii_alphabetic() && c != b'e' && c != b'E' {
            self.i += 1;
            return Some(Some(c));
        }
        Some(None)
    }

    fn num(&mut self) -> Option<f32> {
        self.skip();
        let start = self.i;
        let s = self.s;
        let mut i = self.i;
        if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
            i += 1;
        }
        let mut dot = false;
        while i < s.len() && (s[i].is_ascii_digit() || (s[i] == b'.' && !dot)) {
            dot |= s[i] == b'.';
            i += 1;
        }
        if i < s.len() && (s[i] == b'e' || s[i] == b'E') {
            let mut j = i + 1;
            if j < s.len() && (s[j] == b'+' || s[j] == b'-') {
                j += 1;
            }
            if j < s.len() && s[j].is_ascii_digit() {
                i = j;
                while i < s.len() && s[i].is_ascii_digit() {
                    i += 1;
                }
            }
        }
        self.i = i;
        std::str::from_utf8(&s[start..i]).ok()?.parse().ok()
    }

    /// An arc flag: a single `0` or `1`, which may touch the next number.
    fn flag(&mut self) -> Option<bool> {
        self.skip();
        let c = *self.s.get(self.i)?;
        self.i += 1;
        match c {
            b'0' => Some(false),
            b'1' => Some(true),
            _ => None,
        }
    }
}
