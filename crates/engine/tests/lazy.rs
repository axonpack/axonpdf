//! Children the adapter builds only when they are reached (`"lazy": id`): fetched in batches,
//! counted first when something reads the total, and fetched only once for both passes.

use std::cell::RefCell;
use std::rc::Rc;

use axonpdf::Engine;
use serde_json::{Value, json};

fn row(i: usize) -> Value {
    json!({"type": "TEXT", "props": {}, "children": [{"type": "TEXT_INSTANCE", "value": format!("Row {i}")}]})
}

/// A page with a footer that reads the total, and `list` as its body.
fn doc(list: Value) -> String {
    let footer = json!({"type": "TEXT", "props": {"fixed": true}, "render": 1, "children": []});
    json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [
        {"type": "PAGE", "props": {}, "children": [list, footer]}]})
    .to_string()
}

fn render(engine: &mut Engine, json: &str) -> Vec<u8> {
    let mut out = Vec::new();
    engine
        .render(json, &mut |c| out.extend_from_slice(c))
        .unwrap();
    out
}

fn engine(totals: Rc<RefCell<Vec<Option<u32>>>>) -> Engine {
    let mut engine = Engine::new();
    engine.set_render_callback(Box::new(move |_, a| {
        totals.borrow_mut().push(a.total_pages);
        let text = format!("Page {} of {:?}", a.page_number, a.total_pages);
        json!({"children": [{"type": "TEXT_INSTANCE", "value": text}], "readsTotal": true})
            .to_string()
    }));
    engine
}

#[test]
fn lazy_children_are_counted_and_fetched_once() {
    let rows = 300;
    let totals = Rc::new(RefCell::new(Vec::new()));
    let mut lazy = engine(totals.clone());
    let fetched = Rc::new(RefCell::new(0));
    let n = fetched.clone();
    lazy.set_rows_callback(Box::new(move |id| {
        assert_eq!(id, 9);
        assert!(*n.borrow() <= rows, "asked past the end");
        let mut n = n.borrow_mut();
        let batch: Vec<Value> = (*n..(*n + 64).min(rows)).map(row).collect();
        *n += batch.len();
        json!({"children": batch, "done": *n == rows}).to_string()
    }));
    let pdf = render(
        &mut lazy,
        &doc(json!({"type": "VIEW", "props": {"lazy": 9}, "children": []})),
    );
    // Every batch was fetched once: the pass that drew the pages replayed the counting pass's.
    assert_eq!(*fetched.borrow(), rows);
    assert!(totals.borrow().iter().any(|t| t.is_some()));

    let inline: Vec<Value> = (0..rows).map(row).collect();
    let eager = render(
        &mut engine(Rc::default()),
        &doc(json!({"type": "VIEW", "props": {}, "children": inline})),
    );
    assert_eq!(pdf, eager, "the same file as with the children inline");
}
