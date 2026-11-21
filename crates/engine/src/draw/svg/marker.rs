//! Markers at the start, middle and end vertices of lines, polylines, polygons and paths.

use std::f32::consts::PI;

use super::props::Props;
use super::shapes;
use super::{Ctx, draw_children};
use crate::display::Op;
use crate::path::svg::{Vertex, vertices};
use crate::tree::{Kind, Node};

pub fn draw(ops: &mut Vec<Op>, node: &Node, p: &Props, cx: &Ctx) {
    let Kind::Other(kind) = &node.kind else {
        return;
    };
    let refs = ["markerStart", "markerMid", "markerEnd"]
        .map(|k| p.url(k).and_then(|id| cx.defs.get(id).copied()));
    if refs.iter().all(Option::is_none) {
        return;
    }
    let Some(points) = points(kind, node, p) else {
        return;
    };
    let stroke = p.num("strokeWidth").filter(|w| *w != 0.0).unwrap_or(1.0) as f32;
    let n = points.len();
    for (i, v) in points.iter().enumerate() {
        let which = if i == 0 {
            0
        } else if i + 1 == n {
            2
        } else {
            1
        };
        if let Some(marker) = refs[which] {
            instance(ops, marker, v, stroke, i == 0, cx);
        }
    }
}

fn points(kind: &str, node: &Node, p: &Props) -> Option<Vec<Vertex>> {
    match kind {
        "LINE" => {
            let (x1, y1, x2, y2) = (
                p.n("x1") as f32,
                p.n("y1") as f32,
                p.n("x2") as f32,
                p.n("y2") as f32,
            );
            let angle = (y2 - y1).atan2(x2 - x1);
            Some(vec![
                Vertex {
                    x: x1,
                    y: y1,
                    angle,
                },
                Vertex {
                    x: x2,
                    y: y2,
                    angle,
                },
            ])
        }
        "POLYLINE" | "POLYGON" => Some(list(&shapes::points(p), kind == "POLYGON")),
        "PATH" => Some(vertices(&shapes::outline(node, p)?)),
        _ => None,
    }
}

/// Vertices of a point list; a closed list bisects at its first and last points too.
fn list(pts: &[(f32, f32)], closed: bool) -> Vec<Vertex> {
    let n = pts.len();
    let dir = |a: usize, b: usize| (pts[b].1 - pts[a].1).atan2(pts[b].0 - pts[a].0);
    (0..n)
        .map(|i| {
            let (x, y) = pts[i];
            let angle = match (i, n) {
                (_, 1) => 0.0,
                (0, _) if closed => bisect(dir(n - 1, 0), dir(0, 1)),
                (0, _) => dir(0, 1),
                (i, n) if i + 1 == n && closed => bisect(dir(i - 1, i), dir(i, 0)),
                (i, n) if i + 1 == n => dir(i - 1, i),
                (i, _) => bisect(dir(i - 1, i), dir(i, i + 1)),
            };
            Vertex { x, y, angle }
        })
        .collect()
}

fn bisect(a: f32, b: f32) -> f32 {
    let mut d = b - a;
    while d > PI {
        d -= 2.0 * PI;
    }
    while d < -PI {
        d += 2.0 * PI;
    }
    a + d / 2.0
}

fn instance(ops: &mut Vec<Op>, marker: &Node, v: &Vertex, stroke: f32, start: bool, cx: &Ctx) {
    let p = Props::of(marker, &Default::default(), (1.0, 1.0));
    let (w, h) = (
        p.num("markerWidth").unwrap_or(3.0) as f32,
        p.num("markerHeight").unwrap_or(3.0) as f32,
    );
    let (ref_x, ref_y) = (p.n("refX") as f32, p.n("refY") as f32);
    let angle = match p.get("orient") {
        Some(o) if o.as_str() == Some("auto") => v.angle,
        Some(o) if o.as_str() == Some("auto-start-reverse") => {
            v.angle + if start { PI } else { 0.0 }
        }
        // A fixed angle, in degrees.
        Some(o) => o.as_f64().map_or(0.0, |d| (d as f32).to_radians()),
        None => 0.0,
    };
    ops.push(Op::Save);
    let (s, c) = angle.sin_cos();
    ops.push(Op::Transform([c, s, -s, c, v.x, v.y]));
    if p.str("markerUnits") != Some("userSpaceOnUse") {
        ops.push(Op::Transform([stroke, 0.0, 0.0, stroke, 0.0, 0.0]));
    }
    let view = p.str("viewBox").and_then(super::view_box);
    let container = match view {
        Some([min_x, min_y, vw, vh]) => {
            ops.push(Op::Transform([
                w / vw as f32,
                0.0,
                0.0,
                h / vh as f32,
                0.0,
                0.0,
            ]));
            ops.push(Op::Transform([
                1.0,
                0.0,
                0.0,
                1.0,
                -min_x as f32 - ref_x,
                -min_y as f32 - ref_y,
            ]));
            (vw, vh)
        }
        None => {
            ops.push(Op::Transform([1.0, 0.0, 0.0, 1.0, -ref_x, -ref_y]));
            (w as f64, h as f64)
        }
    };
    draw_children(ops, marker, &Default::default(), container, cx);
    ops.push(Op::Restore);
}
