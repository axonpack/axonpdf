//! Peak heap while rendering long documents, by a counting allocator. Slow in debug builds:
//! `cargo test --release --test heap -- --ignored --nocapture` (`AXONPDF_PAGES` sets the length).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

use axonpdf::Engine;
use serde_json::{Value, json};

struct Counting;
static NOW: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let now = NOW.fetch_add(l.size(), Relaxed) + l.size();
        PEAK.fetch_max(now, Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        NOW.fetch_sub(l.size(), Relaxed);
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static A: Counting = Counting;

fn text(s: String) -> Value {
    json!({"type": "TEXT", "props": {}, "children": [{"type": "TEXT_INSTANCE", "value": s}]})
}

/// A table row of three cells, about 20pt tall.
fn row(i: usize) -> Value {
    let cell = |s: String, w: &str| json!({"type": "VIEW", "props": {}, "style": {"width": w, "padding": 3}, "children": [text(s)]});
    json!({"type": "VIEW", "props": {"wrap": false}, "style": {"flexDirection": "row", "borderBottom": "1 solid #ccc", "fontSize": 10}, "children": [
        cell(format!("SKU-{i}"), "20%"),
        cell(format!("Item number {i} with a longer description"), "60%"),
        cell(format!("{}.00", i % 97), "20%"),
    ]})
}

fn footer() -> Value {
    json!({"type": "TEXT", "props": {"fixed": true}, "style": {"position": "absolute", "bottom": 20, "left": 40, "fontSize": 8}, "render": 1, "children": []})
}

/// `rows` rows in one long `<Page>`, or split over `<Page>`s of `per_page` rows.
fn document(rows: usize, per_page: Option<usize>) -> String {
    let page = |rs: Vec<Value>| json!({"type": "PAGE", "props": {"size": "A4"}, "style": {"padding": 40}, "children": [footer(), {"type": "VIEW", "props": {}, "children": rs}]});
    let all: Vec<Value> = (0..rows).map(row).collect();
    let pages: Vec<Value> = match per_page {
        None => vec![page(all)],
        Some(n) => all.chunks(n).map(|c| page(c.to_vec())).collect(),
    };
    json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": pages}).to_string()
}

fn measure(name: &str, json: String) {
    let mut engine = Engine::new();
    engine.set_render_callback(Box::new(|_, a| {
        json!({"children": [{"type": "TEXT_INSTANCE", "value": format!("Page {} of {:?}", a.page_number, a.total_pages)}], "readsTotal": true}).to_string()
    }));
    let base = NOW.load(Relaxed);
    PEAK.store(base, Relaxed);
    let (mut bytes, mut pages) = (0, 0);
    let start = Instant::now();
    engine
        .render(&json, &mut |b| {
            bytes += b.len();
            pages += b.windows(12).filter(|w| w == b"/Type /Page\n").count();
        })
        .unwrap();
    let peak = PEAK.load(Relaxed) - base;
    println!(
        "{name}: {:.1} s, {} KB of PDF, peak heap above the input {} KB (input JSON {} KB)",
        start.elapsed().as_secs_f64(),
        bytes / 1024,
        peak / 1024,
        json.len() / 1024
    );
}

#[test]
#[ignore]
fn thousand_pages() {
    let pages: usize = std::env::var("AXONPDF_PAGES")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1000);
    // About 37 rows fit a page.
    if std::env::var_os("AXONPDF_ONLY_LONG").is_none() {
        measure(&format!("{pages} <Page>s"), document(pages * 37, Some(37)));
    }
    measure(
        &format!("one <Page> of {pages} pages"),
        document(pages * 37, None),
    );
}

#[test]
fn sizes() {
    println!(
        "Node {} Style {} Line {} LineRun {} Glyph {}",
        size_of::<axonpdf::tree::Node>(),
        size_of::<axonpdf::style::Style>(),
        size_of::<axonpdf::text::Line>(),
        size_of::<axonpdf::text::LineRun>(),
        size_of::<axonpdf::text::Glyph>()
    );
}
