//! The newer pagination engine and Page `layout` templates. The expected boxes are the
//! reference engine's output for the same documents.

use axonpdf::tree::{Kind, Node};
use axonpdf::{Engine, RenderArgs};
use serde_json::{Value, json};

fn walk(n: &Node, top: f64, out: &mut Vec<String>) {
    let top = top + n.layout.top;
    let kind = format!("{:?}", n.kind).to_uppercase();
    out.push(format!("{kind} {} {}", round(top), round(n.layout.height)));
    for c in n.children.iter().filter(|c| c.kind != Kind::TextInstance) {
        walk(c, top, out);
    }
}

fn round(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Lays out `pages`, answering `render` and `layout` ids with `answer`.
fn layout(pages: Value, answer: fn(u32, &RenderArgs) -> Value) -> Vec<Vec<String>> {
    let mut engine = Engine::new();
    engine.set_render_callback(Box::new(move |id, args| {
        json!({"children": answer(id, args), "readsTotal": true}).to_string()
    }));
    let doc = json!({"v": 1, "type": "DOCUMENT", "children": pages}).to_string();
    let mut out = Vec::new();
    engine
        .layout(&doc, &mut |p, _, _| {
            let mut lines = Vec::new();
            p.children.iter().for_each(|c| walk(c, 0.0, &mut lines));
            out.push(lines);
        })
        .unwrap();
    out
}

fn view(style: Value) -> Value {
    json!({"type": "VIEW", "style": style})
}

fn text(s: &str, style: Value) -> Value {
    json!({"type": "TEXT", "style": style, "children": [{"type": "TEXT_INSTANCE", "value": s}]})
}

const LOREM: &str = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua ut enim ad minim veniam quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat";

fn answer(id: u32, a: &RenderArgs) -> Value {
    let n = |x: Option<u32>| x.map_or("undefined".into(), |x| x.to_string());
    match id {
        1 => json!([
            {"type": "VIEW", "style": {"height": 15, "backgroundColor": "red"}, "children": [
                text(&format!("Head {}/{}", a.page_number, n(a.total_pages)), json!({"fontSize": 8}))]},
            {"type": "VIEW", "style": {"flexGrow": 1, "paddingHorizontal": 5}, "children": [{"type": "SLOT"}]},
            view(json!({"height": if a.page_number == 1 { 25 } else { 5 }})),
        ]),
        2 => json!([{"type": "TEXT_INSTANCE", "value": format!("at {} {}/{}", a.page_number,
            n(a.sub_page_number), n(a.sub_page_total_pages))}]),
        _ => json!([{"type": "SLOT"}, {"type": "SLOT"}]),
    }
}

#[test]
fn matches_the_reference() {
    let size = json!([100, 100]);
    let pages = json!([
        {"type": "PAGE", "props": {"size": size, "experimentalPagination": true}, "children": [
            view(json!({"height": 60})),
            {"type": "VIEW", "props": {"minPresenceAhead": 500}, "style": {"height": 30}}]},
        {"type": "PAGE", "props": {"size": size}, "children": [
            view(json!({"height": 60})),
            {"type": "VIEW", "style": {"flexDirection": "row", "alignItems": "flex-end"}, "children": [
                view(json!({"height": 60, "width": 40})), view(json!({"height": 20, "width": 40}))]}]},
        {"type": "PAGE", "props": {"size": size}, "children": [
            {"type": "VIEW", "props": {"fixed": true}, "style": {"height": 20}},
            {"type": "VIEW", "style": {"padding": 5, "borderWidth": 2}, "children": [
                {"type": "VIEW", "props": {"fixed": true}, "style": {"height": 10}},
                view(json!({"height": 30})), view(json!({"height": 30})), view(json!({"height": 30}))]},
            {"type": "VIEW", "props": {"break": true}, "style": {"height": 10}}]},
        {"type": "PAGE", "props": {"size": [200, 120]}, "style": {"padding": 10}, "layout": 1, "children": [
            text(LOREM, json!({"fontSize": 10})),
            view(json!({"height": 50, "backgroundColor": "blue"})),
            {"type": "TEXT", "render": 2}]},
    ]);
    let pages = layout(pages, answer);
    let expected: Vec<Vec<&str>> = vec![
        vec!["VIEW 0 60", "VIEW 60 30"],
        vec!["VIEW 0 60"],
        vec!["VIEW 0 60", "VIEW 0 60", "VIEW 40 20"],
        vec![
            "VIEW 0 20",
            "VIEW 20 80",
            "VIEW 27 10",
            "VIEW 37 30",
            "VIEW 67 30",
            "VIEW 97 3",
        ],
        vec!["VIEW 0 20", "VIEW 20 44", "VIEW 20 10", "VIEW 30 27"],
        vec!["VIEW 0 20", "VIEW 20 10"],
        vec![
            "VIEW 10 15",
            "TEXT 10 8.8",
            "VIEW 25 80",
            "TEXT 25 77",
            "VIEW 102 3",
            "VIEW 105 5",
        ],
        vec![
            "VIEW 10 15",
            "TEXT 10 8.8",
            "VIEW 25 80",
            "VIEW 25 47",
            "TEXT 72 19.8",
            "VIEW 105 5",
        ],
    ];
    assert_eq!(pages, expected);
}

#[test]
fn a_layout_renders_its_children_once() {
    let pages = json!([{"type": "PAGE", "layout": 3, "children": [view(json!({"height": 10}))]}]);
    let doc = json!({"v": 1, "type": "DOCUMENT", "children": pages}).to_string();
    let mut engine = Engine::new();
    engine.set_render_callback(Box::new(|id, a| {
        json!({"children": answer(id, a)}).to_string()
    }));
    let err = engine.layout(&doc, &mut |_, _, _| {}).unwrap_err();
    assert_eq!(
        err,
        "[layout] A page layout must render its children exactly once (found 2)."
    );
}

/// A `fixed` footer written after the content is on every page, not only the last.
#[test]
fn a_fixed_node_after_the_content_repeats() {
    let footer = json!({"type": "VIEW", "props": {"fixed": true},
        "style": {"position": "absolute", "bottom": 0, "height": 10}});
    let block = json!({"type": "VIEW", "props": {"wrap": false}, "style": {"height": 60}});
    let pages = json!([{"type": "PAGE", "props": {"size": [100, 100], "experimentalPagination": true},
        "children": [block.clone(), block.clone(), block, footer]}]);
    let pages = layout(pages, answer);
    assert_eq!(pages.len(), 3);
    assert!(pages.iter().all(|p| p[0] == "VIEW 90 10"), "{pages:?}");
}
