//! Whole documents through the engine: pages cut, numbered and written.

use axonpdf::{Engine, RenderArgs};
use serde_json::{Value, json};

fn render(doc: Value, engine: &mut Engine) -> Vec<u8> {
    let mut pdf = Vec::new();
    engine
        .render(&doc.to_string(), &mut |b| pdf.extend_from_slice(b))
        .unwrap();
    // Set AXONPDF_OUT to a folder to look at the files.
    if let Ok(dir) = std::env::var("AXONPDF_OUT") {
        let name = std::thread::current()
            .name()
            .unwrap_or("out")
            .replace("::", "-");
        std::fs::write(format!("{dir}/{name}.pdf"), &pdf).unwrap();
    }
    pdf
}

fn pages(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf);
    text.matches("/Type /Page\n").count()
        + text.matches("/Type /Page ").count()
        + text.matches("/Type /Page/").count()
}

fn text(s: &str) -> Value {
    json!({"type": "TEXT", "props": {}, "children": [{"type": "TEXT_INSTANCE", "value": s}]})
}

#[test]
fn long_page_wraps_onto_several_pages() {
    let rows: Vec<Value> = (0..120)
        .map(|i| text(&format!("Row {i} of a long list that runs over pages.")))
        .collect();
    let doc = json!({"v": 1, "type": "DOCUMENT", "props": {"title": "Long"}, "children": [
        {"type": "PAGE", "props": {"size": "A4"}, "style": {"padding": 30}, "children": [
            {"type": "VIEW", "props": {"fixed": true}, "style": {"height": 20, "backgroundColor": "#eee"}, "children": []},
            {"type": "VIEW", "props": {}, "children": rows}
        ]}
    ]});
    let pdf = render(doc, &mut Engine::new());
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(pdf.ends_with(b"%%EOF\n") || pdf.ends_with(b"%%EOF"));
    // 120 lines of 19.8pt in 781.89pt of content less the 20pt header: 39 a page.
    assert_eq!(pages(&pdf), 4);
}

#[test]
fn render_props_get_page_numbers_and_the_total() {
    let doc = json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [
        {"type": "PAGE", "props": {}, "children": [
            {"type": "TEXT", "props": {"fixed": true}, "render": 7, "children": []},
            {"type": "VIEW", "props": {"break": true}, "children": []},
            {"type": "VIEW", "props": {"break": true}, "children": []}
        ]}
    ]});
    let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let seen = calls.clone();
    let mut engine = Engine::new();
    engine.set_render_callback(Box::new(move |id, args: &RenderArgs| {
        assert_eq!(id, 7);
        seen.borrow_mut().push(args.clone());
        let label = format!("{} / {:?}", args.page_number, args.total_pages);
        json!({"children": [{"type": "TEXT_INSTANCE", "value": label}], "readsTotal": true})
            .to_string()
    }));
    let pdf = render(doc, &mut engine);
    assert_eq!(pages(&pdf), 3);
    let finals: Vec<(u32, Option<u32>)> = calls
        .borrow()
        .iter()
        .filter(|a| a.sub_page_number.is_some())
        .map(|a| (a.page_number, a.total_pages))
        .collect();
    assert_eq!(
        finals[finals.len() - 3..],
        [(1, Some(3)), (2, Some(3)), (3, Some(3))]
    );
}

/// Long pages keep one Yoga tree between cuts; it must give the boxes a fresh layout gives.
#[test]
fn kept_layout_matches_fresh_layout() {
    let row = |i: usize| {
        let cell = |s: String, style: Value| json!({"type": "VIEW", "props": {}, "style": style, "children": [text(&s)]});
        json!({"type": "VIEW", "props": {"wrap": !i.is_multiple_of(7), "minPresenceAhead": if i.is_multiple_of(11) { 40 } else { 0 }},
               "style": {"flexDirection": "row", "padding": "2 4", "borderBottom": "1 solid #ccc", "fontSize": 9 + (i % 4) as i64},
               "children": [
            cell(format!("Item {i}"), json!({"width": "25%"})),
            cell("A longer description that wraps over a few lines once in a while to make rows of different heights. ".repeat(1 + i % 3), json!({"flex": 1})),
            cell(format!("{}", i * 7), json!({"width": 50, "textAlign": "right"})),
        ]})
    };
    let rows: Vec<Value> = (0..400).map(row).collect();
    let doc = json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [
        {"type": "PAGE", "props": {"size": "A5"}, "style": {"padding": 30}, "children": [
            {"type": "VIEW", "props": {"fixed": true}, "style": {"height": 20}, "children": []},
            {"type": "VIEW", "props": {}, "style": {"border": "1 solid black", "padding": 5}, "children": rows},
            text("The end."),
        ]},
        {"type": "PAGE", "props": {"size": "A6"}, "style": {"padding": 20, "fontSize": 10}, "children":
            (0..300usize).map(|i| text(&format!("Line {i}: some words to fill the line, and more on every fifth. {}", if i.is_multiple_of(5) { "Extra words go here." } else { "" }))).collect::<Vec<_>>()
        }
    ]})
    .to_string();
    let boxes = || {
        let mut out = Vec::new();
        fn walk(n: &axonpdf::tree::Node, out: &mut Vec<[f64; 4]>) {
            out.push([n.layout.left, n.layout.top, n.layout.width, n.layout.height]);
            n.children.iter().for_each(|c| walk(c, out));
        }
        Engine::new()
            .layout(&doc, &mut |p, _, _| walk(p, &mut out))
            .unwrap();
        out
    };
    let kept = boxes();
    // SAFETY: tests in this file do not read the variable concurrently with this write.
    unsafe { std::env::set_var("AXONPDF_FRESH_LAYOUT", "1") };
    let fresh = boxes();
    unsafe { std::env::remove_var("AXONPDF_FRESH_LAYOUT") };
    assert!(kept.len() > 1000);
    let first = kept.iter().zip(&fresh).position(|(a, b)| a != b);
    assert_eq!(
        (kept.len(), first),
        (fresh.len(), None),
        "first difference: {:?}",
        first.map(|i| (kept[i], fresh[i]))
    );
}

#[test]
fn bookmarks_make_an_outline() {
    let doc = json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [
        {"type": "PAGE", "props": {"bookmark": "Chapter"}, "children": [
            {"type": "TEXT", "props": {"bookmark": {"title": "Section", "fit": true}}, "children": [{"type": "TEXT_INSTANCE", "value": "Hi"}]}
        ]}
    ]});
    let pdf = render(doc, &mut Engine::new());
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/Outlines"));
    assert!(text.matches("/Parent").count() >= 2);
}

#[test]
fn hit_slop_enlarges_a_link() {
    let link = |slop: Value| {
        json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [
            {"type": "PAGE", "props": {"size": [200, 200]}, "children": [
                {"type": "LINK", "props": {"src": "https://axonpack.dev", "hitSlop": slop},
                 "style": {"width": 100, "height": 20, "margin": 10}, "children": []}
            ]}
        ]})
    };
    let pdf = render(link(json!({"left": 4, "bottom": 2})), &mut Engine::new());
    assert!(String::from_utf8_lossy(&pdf).contains("/Rect [6 168 110 190]"));
    let pdf = render(link(json!(5)), &mut Engine::new());
    assert!(String::from_utf8_lossy(&pdf).contains("/Rect [5 165 115 195]"));
}
