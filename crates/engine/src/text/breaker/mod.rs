//! Line breaking: Knuth and Plass's total-fit algorithm, with a best-fit fallback.
//!
//! The constants are the ones the components' line breaker uses: line demerits 10, flagged 100,
//! fitness 3000, infinity 10000. Breaks may fall only at glue that follows a box, and at
//! penalties.

mod best_fit;

pub use best_fit::best_fit;

pub const INFINITY: f64 = 10000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
    /// A syllable. `start..end` are character offsets in the paragraph.
    Box { width: f64, start: usize, end: usize },
    /// A run of spaces.
    Glue { width: f64, start: usize, end: usize, stretch: f64, shrink: f64 },
    Penalty { width: f64, penalty: f64, flagged: f64 },
}

#[derive(Clone, Copy, Default)]
pub(super) struct Sum {
    pub(super) width: f64,
    pub(super) stretch: f64,
    pub(super) shrink: f64,
}

struct Breakpoint {
    position: usize,
    demerits: f64,
    line: usize,
    fitness: usize,
    totals: Sum,
    previous: Option<usize>,
}

fn line_length(widths: &[f64], line: usize) -> f64 {
    if line < widths.len() { widths[line - 1] } else { widths[widths.len() - 1] }
}

fn cost(items: &[Item], widths: &[f64], sum: Sum, end: usize, active: &Breakpoint, line: usize) -> f64 {
    let mut width = sum.width - active.totals.width;
    if let Item::Penalty { width: w, .. } = items[end] {
        width += w;
    }
    let length = line_length(widths, line);
    if width < length {
        let stretch = sum.stretch - active.totals.stretch;
        return if stretch > 0.0 { (length - width) / stretch } else { INFINITY };
    }
    if width > length {
        let shrink = sum.shrink - active.totals.shrink;
        return if shrink > 0.0 { (length - width) / shrink } else { INFINITY };
    }
    0.0
}

/// The totals after a break at `index`: past the glue and penalties that follow it.
fn sum_after(items: &[Item], sum: Sum, index: usize) -> Sum {
    let mut r = sum;
    for (i, item) in items.iter().enumerate().skip(index) {
        match *item {
            Item::Glue { width, stretch, shrink, .. } => {
                r.width += width;
                r.stretch += stretch;
                r.shrink += shrink;
            }
            Item::Box { .. } => break,
            Item::Penalty { penalty, .. } if penalty == -INFINITY && i > index => break,
            Item::Penalty { .. } => {}
        }
    }
    r
}

/// Item indices of the breaks, starting with 0. Empty when no set of breaks fits `tolerance`.
pub fn knuth_plass(items: &[Item], widths: &[f64], tolerance: f64) -> Vec<usize> {
    let mut nodes = vec![Breakpoint { position: 0, demerits: 0.0, line: 0, fitness: 0, totals: Sum::default(), previous: None }];
    // The active list, in order, as indices into `nodes`.
    let mut active: Vec<usize> = vec![0];
    let mut sum = Sum::default();

    for (index, item) in items.iter().enumerate() {
        let breakable = match *item {
            Item::Box { width, .. } => {
                sum.width += width;
                false
            }
            Item::Glue { .. } => index > 0 && matches!(items[index - 1], Item::Box { .. }),
            Item::Penalty { penalty, .. } => penalty != INFINITY,
        };
        if breakable {
            consider(items, widths, tolerance, index, sum, &mut nodes, &mut active);
        }
        if let Item::Glue { width, stretch, shrink, .. } = *item {
            sum.width += width;
            sum.stretch += stretch;
            sum.shrink += shrink;
        }
    }

    let mut best: Option<usize> = None;
    for &a in &active {
        if best.is_none_or(|b| nodes[a].demerits < nodes[b].demerits) {
            best = Some(a);
        }
    }
    let mut breaks = Vec::new();
    while let Some(b) = best {
        breaks.push(nodes[b].position);
        best = nodes[b].previous;
    }
    breaks.reverse();
    breaks
}

fn consider(
    items: &[Item],
    widths: &[f64],
    tolerance: f64,
    index: usize,
    sum: Sum,
    nodes: &mut Vec<Breakpoint>,
    active: &mut Vec<usize>,
) {
    let item = items[index];
    let forced = matches!(item, Item::Penalty { penalty, .. } if penalty == -INFINITY);
    let mut i = 0;
    while i < active.len() {
        let mut candidates: [(Option<usize>, f64); 4] = [(None, f64::INFINITY); 4];
        while i < active.len() {
            let a = active[i];
            let line = nodes[a].line + 1;
            let ratio = cost(items, widths, sum, index, &nodes[a], line);
            let removed = ratio < -1.0 || forced;
            if (-1.0..=tolerance).contains(&ratio) {
                let badness = 100.0 * ratio.abs().powi(3);
                let mut demerits = (10.0 + badness) * (10.0 + badness);
                if let Item::Penalty { penalty, flagged, .. } = item {
                    if penalty >= 0.0 {
                        demerits += penalty * penalty;
                    } else if penalty != -INFINITY {
                        demerits -= penalty * penalty;
                    }
                    if let Item::Penalty { flagged: f2, .. } = items[nodes[a].position] {
                        demerits += 100.0 * flagged * f2;
                    }
                }
                let class = if ratio < -0.5 {
                    0
                } else if ratio <= 0.5 {
                    1
                } else if ratio <= 1.0 {
                    2
                } else {
                    3
                };
                if (class as i64 - nodes[a].fitness as i64).abs() > 1 {
                    demerits += 3000.0;
                }
                demerits += nodes[a].demerits;
                if demerits < candidates[class].1 {
                    candidates[class] = (Some(a), demerits);
                }
            }
            if removed {
                active.remove(i);
            } else {
                i += 1;
            }
            if i < active.len() && nodes[active[i]].line >= line && line < widths.len() {
                break;
            }
        }

        let totals = sum_after(items, sum, index);
        for (fitness, (a, demerits)) in candidates.into_iter().enumerate() {
            let Some(a) = a else { continue };
            nodes.push(Breakpoint { position: index, demerits, line: nodes[a].line + 1, fitness, totals, previous: Some(a) });
            active.insert(i, nodes.len() - 1);
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests;
