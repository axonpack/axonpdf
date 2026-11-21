//! Draws an SVG with every element kind into a PDF (`target/tmp/svg.pdf`) and checks it.

use serde_json::json;

use super::draw;
use crate::display::{Op, PageOut};
use crate::draw::Ctx;
use crate::fonts::FontStore;
use crate::images::ImageStore;
use crate::style::Container;
use crate::tree::{Node, RawNode};
use crate::writer::{DocInfo, Writer};

fn svg() -> Node {
    let el = |t: &str, props: serde_json::Value, children: Vec<serde_json::Value>| json!({"type": t, "props": props, "children": children});
    let raw = el(
        "SVG",
        json!({"width": 535, "height": 180, "viewBox": "0 0 535 180"}),
        vec![
            el(
                "DEFS",
                json!({}),
                vec![
                    el(
                        "LINEAR_GRADIENT",
                        json!({"id": "lin", "x1": "0", "y1": "0", "x2": "1", "y2": "0"}),
                        vec![
                            el(
                                "STOP",
                                json!({"offset": "0", "stopColor": "#3a86ff"}),
                                vec![],
                            ),
                            el(
                                "STOP",
                                json!({"offset": "1", "stopColor": "#ff006e", "stopOpacity": 0.6}),
                                vec![],
                            ),
                        ],
                    ),
                    el(
                        "RADIAL_GRADIENT",
                        json!({"id": "rad", "cx": "0.5", "cy": "0.5", "r": "0.5", "fx": "0.3", "fy": "0.3"}),
                        vec![
                            el(
                                "STOP",
                                json!({"offset": "0%", "stopColor": "white"}),
                                vec![],
                            ),
                            el(
                                "STOP",
                                json!({"offset": "100%", "stopColor": "#8338ec"}),
                                vec![],
                            ),
                        ],
                    ),
                    el(
                        "CLIP_PATH",
                        json!({"id": "clip"}),
                        vec![el(
                            "CIRCLE",
                            json!({"cx": "460", "cy": "60", "r": "40"}),
                            vec![],
                        )],
                    ),
                    el(
                        "MARKER",
                        json!({"id": "arrow", "viewBox": "0 0 10 10", "refX": "5", "refY": "5", "markerWidth": "6", "markerHeight": "6", "orient": "auto"}),
                        vec![el(
                            "PATH",
                            json!({"d": "M 0 0 L 10 5 L 0 10 z", "fill": "#333"}),
                            vec![],
                        )],
                    ),
                ],
            ),
            el(
                "RECT",
                json!({"x": "10", "y": "10", "width": "90", "height": "60", "rx": "8", "fill": "url(#lin)", "stroke": "#222", "strokeWidth": 1.5}),
                vec![],
            ),
            el(
                "CIRCLE",
                json!({"cx": "150", "cy": "40", "r": "30", "fill": "url(#rad)"}),
                vec![],
            ),
            el(
                "ELLIPSE",
                json!({"cx": "230", "cy": "40", "rx": "40", "ry": "22", "fill": "#ffbe0b", "fillOpacity": 0.7, "stroke": "#fb5607", "strokeDasharray": "4, 2"}),
                vec![],
            ),
            el(
                "LINE",
                json!({"x1": "280", "y1": "15", "x2": "340", "y2": "65", "stroke": "#06d6a0", "strokeWidth": 6, "strokeLinecap": "round"}),
                vec![],
            ),
            el(
                "POLYLINE",
                json!({"points": "350,65 365,20 380,55 395,15 410,65", "fill": "none", "stroke": "#118ab2", "strokeWidth": 3, "strokeLinejoin": "round"}),
                vec![],
            ),
            el(
                "G",
                json!({"clipPath": "url(#clip)"}),
                vec![
                    el(
                        "RECT",
                        json!({"x": "420", "y": "20", "width": "80", "height": "80", "fill": "#ef476f"}),
                        vec![],
                    ),
                    el(
                        "RECT",
                        json!({"x": "460", "y": "20", "width": "40", "height": "40", "fill": "#073b4c"}),
                        vec![],
                    ),
                ],
            ),
            el(
                "G",
                json!({"transform": "translate(20, 100) rotate(-8)", "opacity": 0.85}),
                vec![
                    el(
                        "POLYGON",
                        json!({"points": "0,60 30,0 60,60", "fill": "#8ac926", "stroke": "#1a1a1a"}),
                        vec![],
                    ),
                    el(
                        "PATH",
                        json!({"d": "M 80 60 C 80 0, 140 0, 140 60 S 200 120, 200 60", "fill": "none", "stroke": "#6a4c93", "strokeWidth": 2}),
                        vec![],
                    ),
                    el(
                        "PATH",
                        json!({"d": "M 220 10 h 50 v 50 h -50 z M 232 22 h 26 v 26 h -26 z", "fill": "#ff595e", "fillRule": "evenodd"}),
                        vec![],
                    ),
                ],
            ),
            el(
                "LINE",
                json!({"x1": "300", "y1": "150", "x2": "380", "y2": "110", "stroke": "#333", "strokeWidth": 1.5, "markerEnd": "url(#arrow)"}),
                vec![],
            ),
            json!({"type": "TEXT", "props": {"x": "400", "y": "140", "fill": "#1d3557", "textAnchor": "start"}, "style": {"fontSize": 14}, "children": [
                {"type": "TEXT_INSTANCE", "value": "Svg text"},
                {"type": "TSPAN", "props": {"fill": "#e63946"}, "style": {"fontFamily": "Helvetica-Bold"}, "children": [{"type": "TEXT_INSTANCE", "value": "tspan"}]},
            ]}),
            json!({"type": "TEXT", "props": {"x": "470", "y": "165", "textAnchor": "middle", "fill": "#666"}, "style": {"fontSize": 9}, "children": [
                {"type": "TEXT_INSTANCE", "value": "anchored middle"},
            ]}),
        ],
    );
    let raw: RawNode = serde_json::from_value(raw).unwrap();
    let mut n = Node::build(raw, &Container::default(), None);
    n.layout.width = 535.0;
    n.layout.height = 180.0;
    n
}

#[test]
fn draws_every_svg_element() {
    let fonts = FontStore::new();
    let mut images = ImageStore::new();
    let lookup = |_: u32| None;
    let cx = Ctx {
        fonts: &fonts,
        images: &lookup,
        paint: &|_, _, _| None,
        outline: &Default::default(),
        forms: Default::default(),
    };
    let mut ops = Vec::new();
    draw(&mut ops, &svg(), 30.0, 30.0, [0.0; 4], &cx);
    let count = |f: fn(&Op) -> bool| ops.iter().filter(|o| f(o)).count();
    assert_eq!(
        count(|o| matches!(o, Op::Path { .. })),
        12,
        "shapes and the arrow head"
    );
    assert_eq!(count(|o| matches!(o, Op::Text(_))), 3);
    assert_eq!(
        count(|o| matches!(o, Op::Save)),
        count(|o| matches!(o, Op::Restore))
    );

    let mut pdf = Vec::new();
    let mut out = |c: &[u8]| pdf.extend_from_slice(c);
    let mut w = Writer::new(DocInfo::default());
    w.begin(&mut out);
    w.write_page(
        &PageOut {
            width: 595.0,
            height: 260.0,
            ops,
        },
        &fonts,
        &mut images,
        &mut out,
    );
    w.finish(&fonts, &mut out);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("svg.pdf"), &pdf).unwrap();
}
