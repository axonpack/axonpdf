//! The adapter's functions the engine calls besides `render`: canvas painters and hyphenation.

use std::cell::RefCell;
use std::collections::HashMap;

/// Runs a canvas's `paint` function by id at a width and height (NaN for both when the canvas
/// is only being measured; the adapter passes `undefined`). Answers the painter calls as a JSON
/// list of `[method, ...args]`.
pub type PaintCallback = Box<dyn FnMut(u32, f32, f32) -> String>;

/// Asks a `Rows` (by its `rows` id) for its next batch: `{"children": [...], "done": bool}`.
pub type RowsCallback = Box<dyn FnMut(u32) -> String>;

/// Runs a hyphenation function by id on one word. Answers the parts as a JSON list of strings.
pub type HyphenationCallback = Box<dyn FnMut(u32, &str) -> String>;

#[derive(Default)]
pub struct Hooks {
    pub paint: RefCell<Option<PaintCallback>>,
    pub hyphenate: RefCell<Option<HyphenationCallback>>,
    pub rows: RefCell<Option<RowsCallback>>,
    /// Words already split, by function: the same word asks once per document.
    words: RefCell<HashMap<(u32, String), Vec<String>>>,
}

impl Hooks {
    pub fn paint(&self, id: u32, width: f64, height: f64) -> Option<String> {
        let mut cb = self.paint.borrow_mut();
        Some(cb.as_mut()?(id, width as f32, height as f32))
    }

    /// The next batch of a `Rows`, or `None` without a callback.
    pub fn rows(&self, id: u32) -> Option<String> {
        Some(self.rows.borrow_mut().as_mut()?(id))
    }

    pub fn hyphenate(&self, id: u32, word: &str) -> Option<Vec<String>> {
        let key = (id, word.to_string());
        if let Some(parts) = self.words.borrow().get(&key) {
            return Some(parts.clone());
        }
        let json = self.hyphenate.borrow_mut().as_mut()?(id, word);
        let parts: Vec<String> = serde_json::from_str(&json).ok()?;
        self.words.borrow_mut().insert(key, parts.clone());
        Some(parts)
    }

    /// Forgets the split words; a new document may bring new functions under the same ids.
    pub fn reset(&self) {
        self.words.borrow_mut().clear();
    }
}
