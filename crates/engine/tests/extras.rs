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
    bands_numbers_and_counts_with(false);
    bands_numbers_and_counts_with(true);
}

/// `next`: with the newer pagination engine.
fn bands_numbers_and_counts_with(next: bool) {
    let footer = json!({"type": "VIEW", "props": {}, "style": {"padding": 10}, "children": [
        {"type": "TEXT", "props": {}, "children": [
            {"type": "TEXT_INSTANCE", "value": "Page "}, {"type": "PAGE_NUMBER", "props": {}, "children": []},
            {"type": "TEXT_INSTANCE", "value": " of "}, {"type": "PAGE_COUNT", "props": {}, "children": []}]}]});
    let header =
        json!({"type": "VIEW", "props": {}, "style": {"height": 40}, "children": [text("Header")]});
    let rows: Vec<Value> = (0..60).map(|i| text(&format!("Row {i}"))).collect();
    let json = doc(
        json!({"type": "PAGE", "props": {"size": "A6", "header": header, "footer": footer, "experimentalPagination": next},
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

/// With rows there is no counting run: the total is left for the writer, and the last page is
/// told it is the last.
#[test]
fn rows_documents_defer_the_total() {
    rows_documents_defer_the_total_with(false);
    rows_documents_defer_the_total_with(true);
}

fn rows_documents_defer_the_total_with(next: bool) {
    let fetched = Rc::new(RefCell::new(0));
    let last = Rc::new(RefCell::new(Vec::new()));
    let mut engine = Engine::new();
    let n = fetched.clone();
    engine.set_rows_callback(Box::new(move |_| {
        let mut n = n.borrow_mut();
        let batch: Vec<Value> = (*n..*n + 50).map(|i| text(&format!("Row {i}"))).collect();
        *n += 50;
        json!({"children": batch, "done": *n >= 200}).to_string()
    }));
    let seen = last.clone();
    engine.set_render_callback(Box::new(move |_, a| {
        // The call made when each page is finished.
        if a.is_last.is_some() {
            seen.borrow_mut().push((a.is_last, a.total_pending));
        }
        json!({"children": [{"type": "TEXT_INSTANCE", "value": "x"}], "readsTotal": true})
            .to_string()
    }));
    let footer = json!({"type": "TEXT", "props": {}, "children": [
        {"type": "TEXT_INSTANCE", "value": "of "}, {"type": "PAGE_COUNT", "props": {}, "children": []}]});
    let json = doc(
        json!({"type": "PAGE", "props": {"footer": footer, "experimentalPagination": next}, "children": [
        {"type": "TEXT", "props": {"fixed": true}, "render": 1, "children": []},
        {"type": "ROWS", "props": {"rows": 1}, "children": []}]}),
    );
    let mut deferred = 0;
    engine
        .layout(&json, &mut |p, _, _| {
            fn walk(n: &Node, count: &mut usize) {
                if let Some(t) = &n.text
                    && let Some(lines) = &t.borrow().lines
                {
                    *count += lines
                        .iter()
                        .flat_map(|l| &l.runs)
                        .filter(|r| r.attrs.deferred)
                        .count();
                }
                n.children.iter().for_each(|c| walk(c, count));
            }
            walk(p, &mut deferred);
        })
        .unwrap();
    let calls = last.borrow();
    assert_eq!(deferred, calls.len());
    assert!(calls.iter().all(|c| c.1));
    assert_eq!(calls.iter().filter(|c| c.0 == Some(true)).count(), 1);
    assert_eq!(calls.last().unwrap().0, Some(true));
}

/// Rows pushed in when asked for, as from an async source, give the same file as rows pulled
/// through the callback.
#[test]
fn pushed_rows_resume_the_render() {
    let batch = |from: usize| {
        let rows: Vec<Value> = (from..from + 50)
            .map(|i| text(&format!("Row {i}")))
            .collect();
        json!({"children": rows, "done": from + 50 >= 1000}).to_string()
    };
    let json = doc(json!({"type": "PAGE", "props": {}, "children": [
        {"type": "ROWS", "props": {"rows": 3}, "children": []}, text("After the rows.")]}));

    let mut pulled = Vec::new();
    let mut engine = Engine::new();
    let next = Rc::new(RefCell::new(0));
    let n = next.clone();
    engine.set_rows_callback(Box::new(move |_| {
        let from = *n.borrow();
        *n.borrow_mut() += 50;
        batch(from)
    }));
    engine
        .render(&json, &mut |b| pulled.extend_from_slice(b))
        .unwrap();

    let mut pushed = Vec::new();
    let mut engine = Engine::new();
    let mut step = engine
        .render_begin(json.clone(), &mut |b| pushed.extend_from_slice(b))
        .unwrap();
    let (mut from, mut waits) = (0, 0);
    while let axonpdf::Step::NeedRows(id) = step {
        assert_eq!(id, 3);
        waits += 1;
        step = engine
            .push_rows(id, &batch(from), &mut |b| pushed.extend_from_slice(b))
            .unwrap();
        from += 50;
    }
    assert_eq!(from, 1000);
    assert!(waits >= 20);
    assert_eq!(pushed, pulled);
}

#[test]
fn table_borders_collapse() {
    let cell = |s: &str| json!({"type": "TABLE_CELL", "props": {}, "style": {"border": "1 solid black", "flex": 1}, "children": [text(s)]});
    let row = || json!({"type": "TABLE_ROW", "props": {}, "style": {"border": "2 solid black"}, "children": [cell("a"), cell("b"), cell("c")]});
    let json = doc(json!({"type": "PAGE", "props": {}, "children": [
        {"type": "TABLE", "props": {}, "style": {"border": "3 solid black"}, "children": [row(), row()]}]}));
    let mut seen = Vec::new();
    Engine::new()
        .layout(&json, &mut |p, _, _| {
            let table = &p.children[0];
            for r in &table.children {
                seen.push((
                    r.layout.border,
                    r.children
                        .iter()
                        .map(|c| c.layout.border)
                        .collect::<Vec<_>>(),
                ));
            }
        })
        .unwrap();
    // The table's 3pt top wins over the first row's 2pt; the second row's top meets the
    // first's bottom; in each row the 2pt left wins over the first cell's, and cells share one
    // border between them.
    assert_eq!(seen[0].0, [0.0, 2.0, 2.0, 2.0]);
    assert_eq!(seen[1].0, [0.0, 2.0, 2.0, 2.0]);
    for (_, cells) in &seen {
        assert_eq!(cells[0], [1.0, 1.0, 1.0, 0.0]);
        assert_eq!(cells[1], [1.0, 1.0, 1.0, 0.0]);
        assert_eq!(cells[2], [1.0, 1.0, 1.0, 0.0]);
    }
}

/// A float that first comes in a later batch of rows still pushes what clears it, and the cut
/// sees that: the cleared text never runs past the bottom of its page.
#[test]
fn floats_in_a_later_batch_of_rows_move_the_cut() {
    let mut engine = Engine::new();
    let fetched = Rc::new(RefCell::new(0));
    let count = fetched.clone();
    engine.set_rows_callback(Box::new(move |_| {
        let mut n = count.borrow_mut();
        let batch: Vec<Value> = (*n..*n + 20)
            .map(|i| {
                if i != 55 {
                    return text(&format!("Row {i}"));
                }
                json!({"type": "VIEW", "props": {}, "children": [
                    {"type": "VIEW", "props": {}, "style": {"float": "left", "width": 50, "height": 300}, "children": []},
                    {"type": "TEXT", "props": {}, "style": {"clear": "left"}, "children": [{"type": "TEXT_INSTANCE", "value": "Cleared"}]}
                ]})
            })
            .collect();
        *n += 20;
        json!({"children": batch, "done": *n == 100}).to_string()
    }));
    let json = doc(json!({"type": "PAGE", "props": {"size": "A6"}, "children": [
        {"type": "ROWS", "props": {"rows": 1, "batchSize": 20}, "children": []}]}));
    let mut bottoms = Vec::new();
    engine
        .layout(&json, &mut |p, _, _| {
            fn walk(n: &Node, y: f64, out: &mut Vec<f64>) {
                let y = y + n.layout.top;
                if n.style.clear.is_some() {
                    out.push(y + n.layout.height);
                }
                n.children.iter().for_each(|c| walk(c, y, out));
            }
            walk(p, 0.0, &mut bottoms);
        })
        .unwrap();
    assert!(!bottoms.is_empty());
    assert!(bottoms.iter().all(|b| *b <= 419.53), "{bottoms:?}");
}
