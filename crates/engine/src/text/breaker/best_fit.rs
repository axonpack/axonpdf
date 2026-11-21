//! The fallback when no tolerance fits: greedy, least bad break per line.

use super::{INFINITY, Item, Sum};

/// Greedy breaking for paragraphs no tolerance fits: each line takes the least bad break
/// before it overflows.
pub fn best_fit(items: &[Item], widths: &[f64]) -> Vec<usize> {
    let mut breaks = vec![0];
    let mut count = 0;
    let mut line = 0;
    let mut sub = items;
    while !sub.is_empty() {
        match next_break(sub, widths, line) {
            Some(b) => {
                count += b;
                breaks.push(count);
                sub = &sub[b + 1..];
                count += 1;
                line += 1;
            }
            None => break,
        }
    }
    breaks
}

fn is_gap(item: &Item) -> bool {
    !matches!(item, Item::Box { .. })
}

fn next_break(sub: &[Item], widths: &[f64], line: usize) -> Option<usize> {
    let mut position: Option<usize> = None;
    let mut min_badness = f64::INFINITY;
    let mut sum = Sum::default();
    let length = widths[line.min(widths.len() - 1)];
    let mut hyphen_width = 0.0;
    for i in 0..sub.len() {
        match sub[i] {
            Item::Box { width, .. } => sum.width += width,
            Item::Glue { width, stretch, shrink, .. } => {
                sum.width += width;
                sum.stretch += stretch;
                sum.shrink += shrink;
            }
            Item::Penalty { .. } => {}
        }
        // The last glue or penalty of the run that starts after this item.
        let mut j = i + 1;
        while j < sub.len() && is_gap(&sub[j]) {
            j += 1;
        }
        hyphen_width = match sub[j - 1] {
            Item::Penalty { width, .. } => width,
            _ => 0.0,
        };
        if sum.width - sum.shrink + hyphen_width > length {
            if position.is_none() {
                let mut j = if i == 0 { 1 } else { i };
                while j < sub.len() && is_gap(&sub[j]) {
                    j += 1;
                }
                position = Some(j - 1);
            }
            break;
        }
        if is_gap(&sub[i]) {
            let (stretch, shrink, penalty) = match sub[i] {
                Item::Glue { stretch, shrink, .. } => (Some(stretch), Some(shrink), 0.0),
                Item::Penalty { penalty, .. } => (None, None, penalty),
                Item::Box { .. } => unreachable!(),
            };
            let ratio = if sum.width < length {
                match stretch.filter(|s| *s != 0.0) {
                    Some(s) if sum.stretch - s > 0.0 => (length - sum.width) / sum.stretch,
                    _ => INFINITY,
                }
            } else if sum.width > length {
                match shrink.filter(|s| *s != 0.0) {
                    Some(s) if sum.shrink - s > 0.0 => (length - sum.width) / sum.shrink,
                    _ => INFINITY,
                }
            } else {
                0.0
            };
            let badness = 100.0 * ratio.abs().powi(3) + penalty;
            if min_badness >= badness {
                position = Some(i);
                min_badness = badness;
            }
        }
    }
    if sum.width - sum.shrink + hyphen_width > length { position } else { None }
}
