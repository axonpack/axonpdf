//! axonpdf's own nodes: bands, page numbers and counts, tables, the CSS break props, and rows
//! that arrive in batches.

use std::cell::RefCell;
use std::rc::Rc;

use axonpdf::Engine;
use axonpdf::tree::Node;
use serde_json::{Value, json};

fn text(s: &str) -> Value {
    json!({"type": "TEXT", "props": {}, "children": [{"type": "TEXT_INSTANCE", "value": s}]})
}

fn doc(page: Value) -> String {
    json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [page]}).to_string()
}

/// Each page's text, in order.
fn pages(engine: &mut Engine, json: &str) -> Vec<String> {
    fn walk(n: &Node, out: &mut String) {
        if let Some(t) = &n.text
            && let Some(lines) = &t.borrow().lines
        {
            for l in lines {
                l.runs.iter().for_each(|r| out.push_str(&r.text));
                out.push('|');
            }
        }
        n.children.iter().for_each(|c| walk(c, out));
    }
    let mut out = Vec::new();
    engine
        .layout(json, &mut |p, _, _| {
            let mut s = String::new();
            walk(p, &mut s);
            out.push(s);
        })
        .unwrap();
    out
}

#[test]
fn bands_numbers_and_counts() {
    let footer = json!({"type": "VIEW", "props": {}, "style": {"padding": 10}, "children": [
        {"type": "TEXT", "props": {}, "children": [
            {"type": "TEXT_INSTANCE", "value": "Page "}, {"type": "PAGE_NUMBER", "props": {}, "children": []},
            {"type": "TEXT_INSTANCE", "value": " of "}, {"type": "PAGE_COUNT", "props": {}, "children": []}]}]});
    let header =
        json!({"type": "VIEW", "props": {}, "style": {"height": 40}, "children": [text("Header")]});
    let rows: Vec<Value> = (0..60).map(|i| text(&format!("Row {i}"))).collect();
    let json = doc(
        json!({"type": "PAGE", "props": {"size": "A6", "header": header, "footer": footer},
        "style": {"padding": 20}, "children": rows}),
    );
    let ps = pages(&mut Engine::new(), &json);
    let n = ps.len();
    assert!(n > 2);
    for (i, p) in ps.iter().enumerate() {
        assert!(p.contains("Header|"), "{p}");
        assert!(p.contains(&format!("Page {} of {n}|", i + 1)), "{p}");
    }
    // The bands push the content in: the first row sits under the header.
    let mut first_top = None;
    Engine::new()
        .layout(&json, &mut |p, _, _| {
            let row = p.children.iter().find(|c| !c.has("__band")).unwrap();
            first_top.get_or_insert(row.layout.top);
        })
        .unwrap();
    assert_eq!(first_top, Some(60.0));
}

#[test]
fn table_header_repeats() {
    let cell = |s: &str| json!({"type": "TABLE_CELL", "props": {}, "style": {"flex": 1}, "children": [text(s)]});
    let mut rows =
        vec![json!({"type": "TABLE_HEADER", "props": {}, "children": [cell("Name"), cell("Qty")]})];
    rows.extend((0..80).map(|i| json!({"type": "TABLE_ROW", "props": {}, "children": [cell(&format!("Item {i}")), cell("1")]})));
    let json = doc(
        json!({"type": "PAGE", "props": {"size": "A6"}, "children": [{"type": "TABLE", "props": {}, "children": rows}]}),
    );
    let ps = pages(&mut Engine::new(), &json);
    assert!(ps.len() > 1);
    assert!(ps.iter().all(|p| p.starts_with("Name|Qty|")), "{ps:?}");
    // Cells of a row sit side by side.
    assert!(ps[0].contains("Item 0|1|"));
}

#[test]
fn css_break_props() {
    let json = doc(json!({"type": "PAGE", "props": {}, "children": [
        text("one"),
        {"type": "VIEW", "props": {"breakBefore": "page"}, "children": [text("two")]},
        {"type": "VIEW", "props": {"breakAfter": "page"}, "children": [text("three")]},
        text("four"),
    ]}));
    assert_eq!(
        pages(&mut Engine::new(), &json),
        ["one|", "two|three|", "four|"]
    );
}

#[test]
fn rows_arrive_in_batches() {
    let total = 3000;
    let fetched = Rc::new(RefCell::new(0));
    let mut engine = Engine::new();
    let count = fetched.clone();
    engine.set_rows_callback(Box::new(move |id| {
        assert_eq!(id, 7);
        let mut n = count.borrow_mut();
        let batch: Vec<Value> = (*n..(*n + 50).min(total))
            .map(|i| text(&format!("Row {i}")))
            .collect();
        *n += batch.len();
        json!({"children": batch, "done": *n == total}).to_string()
    }));
    let json = doc(json!({"type": "PAGE", "props": {}, "children": [
        {"type": "ROWS", "props": {"rows": 7, "batchSize": 50}, "children": []}]}));
    let mut pages = 0;
    let mut fetched_at_first_page = 0;
    engine
        .layout(&json, &mut |_, _, _| {
            pages += 1;
            if pages == 1 {
                fetched_at_first_page = *fetched.borrow();
            }
        })
        .unwrap();
    assert_eq!(*fetched.borrow(), total);
    assert!(
        fetched_at_first_page < 400,
        "{fetched_at_first_page} rows fetched for page 1"
    );
    // 3000 lines of 19.8pt, 42 to an A4 page.
    assert_eq!(pages, 72);
}
