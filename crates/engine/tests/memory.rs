//! The writer keeps only small per-page records (an xref offset per object, a page reference):
//! the live heap grows by well under a kilobyte per repeated page, whatever the page draws.
mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

use axonpdf::display::Op;
use axonpdf::writer::Writer;

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        LIVE.fetch_add(new as isize - l.size() as isize, Ordering::Relaxed);
        unsafe { System.realloc(p, l, new) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn heap_stays_flat_over_a_thousand_pages() {
    let mut a = common::assets();
    // Destinations and bookmarks are document data the caller asked for; leave them out.
    let mut page = common::sample_page(&a, 1);
    page.ops
        .retain(|op| !matches!(op, Op::Dest { .. } | Op::Bookmark(_)));
    let mut written = 0usize;
    let mut out = |chunk: &[u8]| written += chunk.len();
    let mut w = Writer::new(common::info());
    w.begin(&mut out);
    let mut at_50 = 0;
    for n in 1..=1000 {
        w.write_page(&page, &a.fonts, &mut a.images, &mut out);
        if n == 50 {
            at_50 = LIVE.load(Ordering::Relaxed);
        }
    }
    let at_1000 = LIVE.load(Ordering::Relaxed);
    w.finish(&a.fonts, &mut out);
    eprintln!("live heap: page 50 {at_50} B, page 1000 {at_1000} B, output {written} B");
    // A page's content, images and fonts would be tens of kilobytes each if any were kept.
    let per_page = (at_1000 - at_50) / 950;
    assert!(
        per_page < 512,
        "{per_page} B kept per page (page 50: {at_50} B, page 1000: {at_1000} B)"
    );
}
