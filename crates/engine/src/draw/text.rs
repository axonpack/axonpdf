//! A text node's lines: run backgrounds, glyphs, links and decoration lines.

use crate::display::{self, Color, LineCap, LinkTarget, Op, Paint, Seg, Stroke, TextRun};
use crate::text::{Line, LineRun};
use crate::tree::Node;

pub fn target(href: &str) -> LinkTarget {
    match href.strip_prefix('#') {
        Some(name) if !name.is_empty() => LinkTarget::Dest(name.to_string()),
        _ => LinkTarget::Uri(href.to_string()),
    }
}

fn alpha(c: Color, opacity: Option<f64>) -> Color {
    Color {
        a: opacity.map_or(c.a, |o| o as f32),
        ..c
    }
}

/// Draws the lines of a text whose border box starts at (`x`, `y`).
pub fn draw(ops: &mut Vec<Op>, n: &Node, x: f64, y: f64, cx: &super::Ctx) {
    let Some(state) = &n.text else { return };
    let state = state.borrow();
    let Some(lines) = &state.lines else { return };
    let first_y = lines.first().map_or(0.0, |l| l.y);
    let ox = x + n.layout.padding[3] - state.align_offset;
    let oy = y + n.layout.padding[0] - first_y;
    for line in lines {
        draw_line(ops, line, ox + line.x, oy + line.y, cx);
    }
}

fn draw_line(ops: &mut Vec<Op>, line: &Line, lx: f64, ly: f64, cx: &super::Ctx) {
    let baseline = ly + line.ascent;
    let mut pen = lx;
    let last = line.runs.len().saturating_sub(1);
    for (i, run) in line.runs.iter().enumerate() {
        if let Some(bg) = run.attrs.background {
            let w = run.x_advance - if i == last { line.overflow_right } else { 0.0 };
            ops.push(Op::Rect {
                rect: rect(pen, ly, w, line.height),
                radii: [0.0; 4],
                fill: Some(Paint::Solid(bg)),
                stroke: None,
            });
        }
        if let Some(href) = &run.attrs.link {
            ops.push(Op::Link {
                rect: rect(
                    pen,
                    baseline - run.height - run.descent,
                    run.x_advance,
                    run.height,
                ),
                target: target(href),
            });
        }
        if let Some(a) = run.attrs.attachment {
            attachments(ops, run, a, pen, baseline, cx);
        }
        if run.attrs.deferred {
            // The writer sets the page total here once it knows it.
            let t = glyphs(run, pen, baseline);
            ops.push(Op::Deferred(display::Deferred {
                key: display::DeferredKey::TotalPages,
                font: t.font,
                size: t.size,
                color: t.color,
                x: t.x,
                baseline: t.y,
                width: run.x_advance as f32,
                align: match run.attrs.align.as_str() {
                    "center" => display::TextAlign::Center,
                    "right" => display::TextAlign::Right,
                    _ => display::TextAlign::Left,
                },
            }));
        } else {
            ops.push(Op::Text(glyphs(run, pen, baseline)));
        }
        pen += run.x_advance;
    }
    decorations(ops, line, lx, ly);
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> display::Rect {
    display::Rect {
        x: x as f32,
        y: y as f32,
        w: w as f32,
        h: h as f32,
    }
}

fn glyphs(run: &LineRun, x: f64, y: f64) -> TextRun {
    let offsets: Vec<usize> = run.text.char_indices().map(|(b, _)| b).collect();
    let glyphs = run
        .glyphs
        .iter()
        .map(|g| display::Glyph {
            id: g.id,
            cluster: offsets
                .get(g.index - run.start)
                .copied()
                .unwrap_or(run.text.len()) as u32,
            x_advance: g.advance as f32,
            x_offset: g.x_offset as f32,
            y_offset: g.y_offset as f32,
        })
        .collect();
    TextRun {
        font: run.font,
        size: run.attrs.size as f32,
        color: alpha(run.attrs.color, run.attrs.opacity),
        x: x as f32,
        y: y as f32,
        glyphs,
        text: run.text.clone(),
    }
}

/// Underlines sit two thicknesses below the baseline, strike-throughs a third of the run's
/// ascent above it; thickness is a point per 12pt of font size, at least half a point.
fn decorations(ops: &mut Vec<Op>, line: &Line, lx: f64, ly: f64) {
    let max_x = line.x_advance - line.overflow_right;
    let mut x = line.overflow_left;
    for run in &line.runs {
        let width = (max_x - x).min(run.x_advance);
        let thickness = (run.attrs.size / 12.0).floor().max(0.5);
        let mut lines = Vec::new();
        if run.attrs.underline {
            lines.push(line.ascent + thickness * 2.0);
        }
        if run.attrs.strike {
            lines.push(line.ascent - run.ascent / 3.0);
        }
        for dy in lines {
            let style = &run.attrs.decoration_style;
            let dash = if style.contains("dashed") {
                vec![3.0 * thickness as f32]
            } else if style.contains("dotted") {
                vec![thickness as f32]
            } else {
                vec![]
            };
            let (x0, y0) = ((lx + x) as f32, (ly + dy) as f32);
            ops.push(Op::Path {
                path: vec![Seg::Move(x0, y0), Seg::Line(x0 + width as f32, y0)],
                fill: None,
                stroke: Some(Stroke {
                    paint: Paint::Solid(alpha(run.attrs.decoration_color, run.attrs.opacity)),
                    width: thickness as f32,
                    cap: LineCap::Butt,
                    join: Default::default(),
                    miter_limit: 10.0,
                    dash,
                    dash_phase: 0.0,
                }),
            });
        }
        x += width;
    }
}

/// Images standing in for U+FFFC: each ends at the pen after its slot, sits on the baseline
/// (moved down by its offset) and fits its box, centred, bottom aligned.
fn attachments(
    ops: &mut Vec<Op>,
    run: &LineRun,
    a: crate::text::Attachment,
    x: f64,
    baseline: f64,
    cx: &super::Ctx,
) {
    let Some((id, (iw, ih))) = (cx.images)(a.asset) else {
        return;
    };
    let chars: Vec<char> = run.text.chars().collect();
    let mut pen = x;
    for g in &run.glyphs {
        pen += g.advance;
        if chars.get(g.index - run.start) != Some(&'\u{fffc}') {
            continue;
        }
        let scale = (a.width / iw).min(a.height / ih);
        let (w, h) = (iw * scale, ih * scale);
        let left = pen - a.width + (a.width - w) / 2.0;
        let top = baseline - a.height + a.y_offset + (a.height - h);
        ops.push(Op::Image {
            id,
            rect: display::Rect {
                x: left as f32,
                y: top as f32,
                w: w as f32,
                h: h as f32,
            },
            opacity: 1.0,
        });
    }
}
