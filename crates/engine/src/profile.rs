//! Where a render's time goes, when profiling is on: each phase's own time (a phase entered
//! inside another pauses it) and each page's time to its last byte. Off, a mark is one load.
//!
//! ponytail: one record per thread, shared by every engine on it; give each engine its own when
//! two profiled renders need to run interleaved.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::time::Instant;

static ON: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
pub enum Phase {
    /// Reading the tree JSON.
    Parse,
    /// Styles, boxes and cutting pages: what isn't one of the others.
    Layout,
    /// Shaping text and breaking it into lines.
    Text,
    /// Turning laid-out pages into drawing operations.
    Paint,
    /// Writing PDF objects and streams.
    Write,
    /// The adapter's functions: `render`, Canvas `paint`, hyphenation, `Rows` batches.
    Callbacks,
}

const NAMES: [&str; 6] = ["parse", "layout", "text", "paint", "write", "callbacks"];

struct Record {
    last: Instant,
    current: Option<Phase>,
    totals: [f64; 6],
    page_start: Instant,
    pages: Vec<f64>,
}

thread_local! {
    static RECORD: RefCell<Option<Record>> = const { RefCell::new(None) };
}

/// Turns profiling on, from nothing measured, or off.
pub fn set(on: bool) {
    ON.store(on, Relaxed);
    let now = Instant::now();
    RECORD.with(|r| {
        *r.borrow_mut() = on.then(|| Record {
            last: now,
            current: None,
            totals: [0.0; 6],
            page_start: now,
            pages: Vec::new(),
        })
    });
}

/// Times what runs until the returned guard drops as `phase`.
pub fn enter(phase: Phase) -> Enter {
    Enter(ON.load(Relaxed).then(|| switch(Some(phase))))
}

#[must_use]
pub struct Enter(Option<Option<Phase>>);

impl Drop for Enter {
    fn drop(&mut self) {
        if let Some(outer) = self.0 {
            switch(outer);
        }
    }
}

/// Ends the running phase's stretch and starts `to`'s; returns the one that was running.
fn switch(to: Option<Phase>) -> Option<Phase> {
    let now = Instant::now();
    RECORD.with(|r| {
        let mut r = r.borrow_mut();
        let r = r.as_mut()?;
        if let Some(p) = r.current {
            r.totals[p as usize] += ms(now - r.last);
        }
        r.last = now;
        std::mem::replace(&mut r.current, to)
    })
}

/// A page's last byte is written.
pub fn page() {
    if !ON.load(Relaxed) {
        return;
    }
    let now = Instant::now();
    RECORD.with(|r| {
        if let Some(r) = r.borrow_mut().as_mut() {
            r.pages.push(ms(now - r.page_start));
            r.page_start = now;
        }
    });
}

/// What was measured since profiling was turned on, in milliseconds:
/// `{"parse": .., "layout": .., ..., "pages": [..]}`.
pub fn report() -> String {
    RECORD.with(|r| {
        let r = r.borrow();
        let mut out = serde_json::Map::new();
        if let Some(r) = r.as_ref() {
            for (name, ms) in NAMES.iter().zip(r.totals) {
                out.insert((*name).into(), ms.into());
            }
            out.insert("pages".into(), r.pages.clone().into());
        }
        serde_json::Value::Object(out).to_string()
    })
}

fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_phases_pause_outer_ones() {
        set(true);
        {
            let _layout = enter(Phase::Layout);
            std::thread::sleep(std::time::Duration::from_millis(5));
            let _text = enter(Phase::Text);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        page();
        let r: serde_json::Value = serde_json::from_str(&report()).unwrap();
        set(false);
        let (layout, text) = (r["layout"].as_f64().unwrap(), r["text"].as_f64().unwrap());
        assert!(
            (5.0..10.0).contains(&layout) && (5.0..10.0).contains(&text),
            "{r}"
        );
        assert_eq!(r["pages"].as_array().unwrap().len(), 1);
    }
}
