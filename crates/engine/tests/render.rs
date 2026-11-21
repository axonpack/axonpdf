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
