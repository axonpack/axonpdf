//! An `Svg` node's content into display ops: the viewBox mapping, then each child with its
//! inherited presentation props, clip path, transform, fill or gradient, stroke and markers.

mod gradient;
mod marker;
mod props;
mod shapes;
mod text;

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::display::{Color, Fill, Op, Paint, Rect, Stroke};
use crate::path::rounded_rect;
use crate::style::transform::{self, Transform};
use crate::tree::{Kind, Node};
use props::Props;

pub struct Ctx<'a> {
    defs: HashMap<&'a str, &'a Node>,
    draw: &'a super::Ctx<'a>,
}

/// Draws the content of `svg`, whose border box is at (`x`, `y`) with corner `radii`.
pub fn draw(ops: &mut Vec<Op>, svg: &Node, x: f64, y: f64, radii: [f32; 4], draw: &super::Ctx) {
    let l = &svg.layout;
    if l.width <= 0.0 || l.height <= 0.0 {
        return;
    }
    let mut defs = HashMap::new();
    for d in svg.children.iter().filter(|c| is(c, "DEFS")) {
        for def in &d.children {
            if let Some(id) = def.props.get("id").and_then(Value::as_str) {
                defs.insert(id, def);
            }
        }
    }
    let cx = Ctx { defs, draw };
    let view = svg
        .props
        .get("viewBox")
        .and_then(Value::as_str)
        .and_then(view_box);
    let num = |k: &str| svg.props.get(k).and_then(crate::style::units::parse_float);
    let container = view.map_or(
        (
            num("width").unwrap_or(l.width),
            num("height").unwrap_or(l.height),
        ),
        |v| (v[2], v[3]),
    );
    let root = Props::of(svg, &Map::new(), container);
    ops.push(Op::Save);
    let rect = Rect {
        x: x as f32,
        y: y as f32,
        w: l.width as f32,
        h: l.height as f32,
    };
    ops.push(Op::Clip {
        path: rounded_rect(rect, radii),
        rule: Default::default(),
    });
    ops.push(translate(x + l.padding[3], y + l.padding[0]));
    if let Some(v) = view {
        let aspect = svg
            .props
            .get("preserveAspectRatio")
            .and_then(Value::as_str)
            .unwrap_or("");
        ops.extend(fit(v, l.width, l.height, aspect));
    }
    draw_children(ops, svg, &root.inherited(false), container, &cx);
    ops.push(Op::Restore);
}

fn draw_children(
    ops: &mut Vec<Op>,
    parent: &Node,
    inherited: &Map<String, Value>,
    container: (f64, f64),
    cx: &Ctx,
) {
    for child in &parent.children {
        if is(child, "DEFS") || child.kind == Kind::TextInstance {
            continue;
        }
        let p = Props::of(child, inherited, container);
        ops.push(Op::Save);
        if let Some(clip) = p.url("clipPath").and_then(|id| cx.defs.get(id)) {
            let path = clip
                .children
                .iter()
                .filter_map(|c| shapes::outline(c, &Props::of(c, &Map::new(), container)))
                .flatten()
                .collect();
            ops.push(Op::Clip {
                path,
                rule: Default::default(),
            });
        }
        let ops_of = child
            .style
            .transform
            .clone()
            .or_else(|| p.get("transform").and_then(transform::parse));
        for t in ops_of.unwrap_or_default() {
            ops.push(Op::Transform(matrix(&t)));
        }
        draw_node(ops, child, &p, cx);
        marker::draw(ops, child, &p, cx);
        if child.kind != Kind::Text {
            draw_children(ops, child, &p.inherited(false), container, cx);
        }
        ops.push(Op::Restore);
    }
}

fn draw_node(ops: &mut Vec<Op>, node: &Node, p: &Props, cx: &Ctx) {
    if node.kind == Kind::Text {
        return text::draw(ops, node, p, cx.draw.fonts);
    }
    if node.kind == Kind::Image {
        let id = node
            .props
            .get("href")
            .or(node.props.get("src"))
            .and_then(|v| v.get("asset"))
            .and_then(Value::as_u64);
        if let Some((id, _)) = id.and_then(|id| (cx.draw.images)(id as u32)) {
            let rect = Rect {
                x: p.n("x") as f32,
                y: p.n("y") as f32,
                w: p.n("width") as f32,
                h: p.n("height") as f32,
            };
            ops.push(Op::Image {
                id,
                rect,
                opacity: p.num("opacity").unwrap_or(1.0) as f32,
            });
        }
        return;
    }
    let Some(path) = shapes::outline(node, p) else {
        return;
    };
    let (fill_alpha, stroke_alpha) = p.alphas();
    let alpha = |c: Color, a: f32| Paint::Solid(Color { a: c.a * a, ..c });
    let mut gradient_alpha = None;
    let fill = p.is("fill").then(|| {
        let paint = match p.url("fill").and_then(|id| cx.defs.get(id)) {
            Some(def) => {
                gradient_alpha = Some(fill_alpha);
                gradient::paint(def, shapes::bbox(node, p)).unwrap_or(Paint::Solid(Color::BLACK))
            }
            None => alpha(p.color("fill").unwrap_or(Color::BLACK), fill_alpha),
        };
        Fill {
            paint,
            rule: p.fill_rule(),
        }
    });
    let stroke = p.is("stroke").then(|| Stroke {
        paint: alpha(p.color("stroke").unwrap_or(Color::BLACK), stroke_alpha),
        width: p.num("strokeWidth").filter(|w| *w != 0.0).unwrap_or(1.0) as f32,
        cap: p.cap(),
        join: p.join(),
        miter_limit: 10.0,
        dash: p.dash(),
        dash_phase: 0.0,
    });
    if let Some(a) = gradient_alpha.filter(|a| *a < 1.0) {
        ops.extend([Op::Save, Op::Opacity(a)]);
        ops.push(Op::Path { path, fill, stroke });
        ops.push(Op::Restore);
    } else if fill.is_some() || stroke.is_some() {
        ops.push(Op::Path { path, fill, stroke });
    }
}

fn is(n: &Node, kind: &str) -> bool {
    matches!(&n.kind, Kind::Other(k) if k == kind)
}

fn translate(x: f64, y: f64) -> Op {
    Op::Transform([1.0, 0.0, 0.0, 1.0, x as f32, y as f32])
}

/// `"minX minY width height"`.
pub fn view_box(s: &str) -> Option<[f64; 4]> {
    let v: Vec<f64> = s
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(|p| p.parse().unwrap_or(f64::NAN))
        .collect();
    v.try_into().ok()
}

/// One SVG transform operation (degrees) as a matrix.
fn matrix(t: &Transform) -> [f32; 6] {
    let v = |i: usize| t.values.get(i).copied().unwrap_or(0.0);
    let m: [f64; 6] = match t.op.as_str() {
        "scale" => [v(0), 0.0, 0.0, v(1), 0.0, 0.0],
        "translate" => [1.0, 0.0, 0.0, 1.0, v(0), v(1)],
        "rotate" => {
            let (s, c) = v(0).to_radians().sin_cos();
            let (cx, cy) = (v(1), v(2));
            [c, s, -s, c, cx - c * cx + s * cy, cy - s * cx - c * cy]
        }
        "skew" => [
            1.0,
            v(1).to_radians().tan(),
            v(0).to_radians().tan(),
            1.0,
            0.0,
            0.0,
        ],
        "matrix" => [v(0), v(1), v(2), v(3), v(4), v(5)],
        _ => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };
    m.map(|x| x as f32)
}

/// The viewBox mapping for `preserveAspectRatio` (`align` and `meet` or `slice`).
fn fit(v: [f64; 4], width: f64, height: f64, aspect: &str) -> Vec<Op> {
    let [x, y, lw, lh] = v;
    let mut words = aspect.trim().trim_start_matches("defer").split_whitespace();
    let align = words.next().unwrap_or("xMidYMid");
    let slice = words.next() == Some("slice");
    let scale = |s: f64| Op::Transform([s as f32, 0.0, 0.0, s as f32, 0.0, 0.0]);
    if align == "none" {
        return vec![
            Op::Transform([
                (width / lw) as f32,
                0.0,
                0.0,
                (height / lh) as f32,
                0.0,
                0.0,
            ]),
            translate(-x, -y),
        ];
    }
    let (x_part, y_part) = (
        align.get(1..4).unwrap_or("Mid"),
        align.get(5..8).unwrap_or("Mid"),
    );
    let share = |part: &str| match part {
        "Min" => 0.0,
        "Mid" => 0.5,
        _ => 1.0,
    };
    if (lw / lh < width / height) != slice {
        // Height decides the scale; the slack is horizontal.
        let slack = lw - width * lh / height;
        vec![
            scale(height / lh),
            translate(-x - slack * share(x_part), -y),
        ]
    } else {
        let slack = lh - height * lw / width;
        vec![scale(width / lw), translate(-x, -y - slack * share(y_part))]
    }
}

#[cfg(test)]
mod tests;
