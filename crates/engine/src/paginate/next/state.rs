//! What a page being filled keeps track of.

use std::rc::Rc;

use super::items::{FillResult, Fragment, Placed, Step, is_repeat_placed};

/// A lazy item rendered on this page, put back if nothing of it stays here.
pub struct Pending {
    pub at: usize,
    pub lazy: Fragment,
    pub count: usize,
    pub placed_then: usize,
}

pub struct State {
    pub placed: Vec<Placed>,
    pub used: f64,
    /// The last legal break: where the rest starts, and how many placements stay.
    pub best: Option<(usize, usize)>,
    pub pending: Option<Pending>,
    pub page: u32,
    pub height: f64,
    /// May place something that does not fit, at the top of an empty page.
    pub can_force: bool,
    /// Real content sits above this fill on the page.
    pub content_above: bool,
    /// No break may fall above this.
    pub forbid_until: f64,
}

impl State {
    pub fn done(&mut self, remaining: Vec<Fragment>) -> Step {
        Step::Done(FillResult {
            placed: std::mem::take(&mut self.placed),
            remaining,
        })
    }

    /// Puts back a lazy rendered on this page when nothing of it was placed: its index.
    pub fn restore(&mut self, frags: &mut Vec<Fragment>) -> Option<usize> {
        let p = self.pending.take()?;
        if self.placed.len() > p.placed_then {
            return None;
        }
        frags.splice(p.at..p.at + p.count, [p.lazy]);
        Some(p.at)
    }

    /// Something other than repeating content is already above this point.
    pub fn content_above(&self, frags: &[Fragment]) -> bool {
        let from_repeat = |p: &Placed| {
            frags.iter().any(|f| {
                f.origin.as_ref().is_some_and(|o| o.is_repeat()) && Rc::ptr_eq(&f.item, &p.item)
            })
        };
        self.content_above
            || self
                .placed
                .iter()
                .any(|p| !is_repeat_placed(p) && !from_repeat(p))
    }
}
