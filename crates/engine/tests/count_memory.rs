//! A long page whose footer reads the total is cut twice, once to count the pages: the peak heap
//! must not grow with the page count, whether its rows come inline or in lazy batches.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, Ordering::Relaxed};

use axonpdf::Engine;
use serde_json::{Value, json};

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        PEAK.fetch_max(
            LIVE.fetch_add(l.size() as isize, Relaxed) + l.size() as isize,
            Relaxed,
        );
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as isize, Relaxed);
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static A: Counting = Counting;

/// A table row about 20pt tall, so about 37 fit a page.
fn row(i: usize) -> Value {
    let cell = |s: String| json!({"type": "TEXT", "props": {}, "style": {"width": "25%"}, "children": [{"type": "TEXT_INSTANCE", "value": s}]});
    json!({"type": "VIEW", "props": {"wrap": false}, "style": {"flexDirection": "row", "height": 20}, "children": [
        cell(format!("Row {i}")), cell(format!("Item number {i}")), cell(format!("{}.00", i % 97)), cell("ok".into())
    ]})
}

/// One `<Page>` of `rows` rows, in lazy batches or inline, and a footer that reads the total.
fn document(rows: usize, lazy: bool) -> String {
    let footer = json!({"type": "TEXT", "props": {"fixed": true}, "style": {"position": "absolute", "bottom": 20}, "render": 1, "children": []});
    let list = if lazy {
        let head = json!({"type": "VIEW", "props": {"fixed": true}, "style": {"flexDirection": "row", "height": 20}, "children": []});
        json!({"type": "VIEW", "props": {"lazy": 9}, "style": {"borderTopWidth": 0.5}, "children": [head]})
    } else {
        json!({"type": "VIEW", "props": {}, "children": (0..rows).map(row).collect::<Vec<_>>()})
    };
    json!({"v": 1, "type": "DOCUMENT", "props": {}, "children": [
        {"type": "PAGE", "props": {}, "style": {"padding": 40}, "children": [list, footer]}]})
    .to_string()
}

/// The peak heap while rendering `rows` rows, above what was live before (the input included,
/// which the engine reads in place).
fn peak(rows: usize, lazy: bool) -> isize {
    let mut engine = Engine::new();
    engine.set_render_callback(Box::new(|_, a| {
        let text = format!("Page {} of {:?}", a.page_number, a.total_pages);
        json!({"children": [{"type": "TEXT_INSTANCE", "value": text}], "readsTotal": true})
            .to_string()
    }));
    let next = Rc::new(Cell::new(0));
    let n = next.clone();
    engine.set_rows_callback(Box::new(move |_| {
        let from = n.get();
        let batch: Vec<Value> = (from..(from + 64).min(rows)).map(row).collect();
        n.set(from + batch.len());
        json!({"children": batch, "done": n.get() == rows}).to_string()
    }));
    let json = document(rows, lazy);
    let base = LIVE.load(Relaxed);
    PEAK.store(base, Relaxed);
    engine.render_owned(json, &mut |_| {}).unwrap();
    PEAK.load(Relaxed) - base
}

#[test]
fn counting_the_pages_keeps_the_heap_flat() {
    for lazy in [false, true] {
        // About 20 and 200 pages.
        let (short, long) = (peak(740, lazy), peak(7400, lazy));
        let per_page = (long - short) / 180;
        eprintln!("lazy {lazy}: peak {short} B at 20 pages, {long} B at 200 pages");
        // The lazy batches the count fetched are kept, deflated, for the second pass: about
        // 0.6 KB a page here. Anything kept whole per page is tens of kilobytes.
        let limit = if lazy { 2048 } else { 512 };
        assert!(
            per_page < limit,
            "lazy {lazy}: {per_page} B more peak heap per page"
        );
    }
}
