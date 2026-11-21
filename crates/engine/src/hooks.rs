//! The adapter's functions the engine calls besides `render`: canvas painters and hyphenation.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};

use serde_json::value::RawValue;

use crate::profile::{self, Phase};

/// Runs a canvas's `paint` function by id at a width and height (NaN for both when the canvas
/// is only being measured; the adapter passes `undefined`). Answers the painter calls as a JSON
/// list of `[method, ...args]`.
pub type PaintCallback = Box<dyn FnMut(u32, f32, f32) -> String>;

/// Asks a `Rows` (by its `rows` id) for its next batch: `{"children": [...], "done": bool}`.
pub type RowsCallback = Box<dyn FnMut(u32) -> String>;

/// Runs a hyphenation function by id on one word. Answers the parts as a JSON list of strings.
pub type HyphenationCallback = Box<dyn FnMut(u32, &str) -> String>;

/// Rows pushed in and not taken yet, and whether the last batch has come.
pub type Pushed = (Vec<Box<RawValue>>, bool);

#[derive(Default)]
pub struct Hooks {
    pub paint: RefCell<Option<PaintCallback>>,
    pub hyphenate: RefCell<Option<HyphenationCallback>>,
    pub rows: RefCell<Option<RowsCallback>>,
    /// Batches pushed in by the adapter for a resumable render, by `rows` id, and whether the
    /// last one has come.
    pub pushed: RefCell<HashMap<u32, Pushed>>,
    /// Words already split, by function: the same word asks once per document.
    words: RefCell<HashMap<(u32, String), Vec<String>>>,
    replay: RefCell<Replay>,
}

/// Batches fetched while the pages are counted, kept deflated so the pass that draws them
/// replays them instead of asking again: the adapter builds each batch once.
#[derive(Default)]
struct Replay {
    recording: bool,
    batches: HashMap<u32, VecDeque<Vec<u8>>>,
}

impl Hooks {
    pub fn paint(&self, id: u32, width: f64, height: f64) -> Option<String> {
        let _time = profile::enter(Phase::Callbacks);
        let mut cb = self.paint.borrow_mut();
        Some(cb.as_mut()?(id, width as f32, height as f32))
    }

    /// The next batch of a `Rows`: from its callback, else what was pushed. `Err(WAIT…)` when
    /// nothing was pushed yet and more is to come.
    pub fn rows(&self, id: u32) -> Result<(Vec<Box<RawValue>>, bool), String> {
        let replayed = {
            let mut replay = self.replay.borrow_mut();
            let kept = (!replay.recording)
                .then(|| replay.batches.get_mut(&id)?.pop_front())
                .flatten();
            kept.map(|bytes| String::from_utf8(decompress_to_vec(&bytes).unwrap()).unwrap())
        };
        let fetched = || {
            let mut cb = self.rows.borrow_mut();
            let cb = cb.as_mut()?;
            let _time = profile::enter(Phase::Callbacks);
            Some(cb(id))
        };
        if let Some(json) = replayed.or_else(fetched) {
            let mut replay = self.replay.borrow_mut();
            if replay.recording {
                let deflated = compress_to_vec(json.as_bytes(), 1);
                replay.batches.entry(id).or_default().push_back(deflated);
            }
            let b: Batch = serde_json::from_str(&json).map_err(|e| format!("rows {id}: {e}"))?;
            return Ok((b.children, b.done));
        }
        let mut pushed = self.pushed.borrow_mut();
        let entry = pushed.entry(id).or_default();
        if entry.0.is_empty() && !entry.1 {
            return Err(format!("{WAIT}{id}"));
        }
        Ok((std::mem::take(&mut entry.0), entry.1))
    }

    /// Adds a batch pushed in for `Rows` `id`: `{"children": [...], "done": bool}`.
    pub fn push_rows(&self, id: u32, json: &str) -> Result<(), String> {
        let b: Batch = serde_json::from_str(json).map_err(|e| format!("rows {id}: {e}"))?;
        let mut pushed = self.pushed.borrow_mut();
        let entry = pushed.entry(id).or_default();
        entry.0.extend(b.children);
        entry.1 |= b.done;
        Ok(())
    }

    pub fn hyphenate(&self, id: u32, word: &str) -> Option<Vec<String>> {
        let key = (id, word.to_string());
        if let Some(parts) = self.words.borrow().get(&key) {
            return Some(parts.clone());
        }
        let json = {
            let _time = profile::enter(Phase::Callbacks);
            self.hyphenate.borrow_mut().as_mut()?(id, word)
        };
        let parts: Vec<String> = serde_json::from_str(&json).ok()?;
        self.words.borrow_mut().insert(key, parts.clone());
        Some(parts)
    }

    /// Forgets the split words and kept batches; a new document may bring new functions and
    /// rows under the same ids.
    pub fn reset(&self) {
        self.words.borrow_mut().clear();
        *self.replay.borrow_mut() = Replay::default();
    }

    /// While on, every batch fetched is kept for [`Hooks::rows`] to hand out again, in order.
    pub fn record(&self, on: bool) {
        self.replay.borrow_mut().recording = on;
    }
}

/// Starts the error a cut gives when it must wait for rows; the `rows` id follows.
pub const WAIT: &str = "\u{0}waiting for rows ";

/// The `rows` id an error waits for, if it is a wait.
pub fn waiting_for(error: &str) -> Option<u32> {
    error.strip_prefix(WAIT)?.parse().ok()
}

/// A batch of rows.
#[derive(serde::Deserialize)]
struct Batch {
    #[serde(default)]
    children: Vec<Box<RawValue>>,
    #[serde(default)]
    done: bool,
}
