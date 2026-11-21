//! Filling one page: items go down the page in order, each remembering the last place a break
//! may fall, until one does not fit.

use std::rc::Rc;

use super::items::{
    FillResult, Fragment, Item, Materialize, Placed, Step, height, reach, to_fragments,
};
use super::state::{Pending, State};
use super::{overflow, row};

/// Fills `frags` into `height`. `frags` is changed in place as lazy items are rendered.
pub fn fill(
    frags: &mut Vec<Fragment>,
    height: f64,
    page: u32,
    can_force: bool,
    content_above: bool,
    mx: &mut Materialize,
) -> Result<FillResult, String> {
    let mut st = State {
        placed: Vec::new(),
        used: 0.0,
        best: None,
        pending: None,
        page,
        height,
        can_force,
        content_above,
        forbid_until: 0.0,
    };
    let mut i = 0;
    while i < frags.len() {
        match step(&mut st, frags, i, mx)? {
            Step::Done(r) => return Ok(r),
            Step::Rewind => {}
            _ => i += 1,
        }
    }
    Ok(FillResult {
        placed: st.placed,
        remaining: Vec::new(),
    })
}

fn step(
    st: &mut State,
    frags: &mut Vec<Fragment>,
    i: usize,
    mx: &mut Materialize,
) -> Result<Step, String> {
    let item = frags[i].item.clone();
    match &*item {
        Item::Penalty { force: true, .. } => Ok(st.done(frags[i + 1..].to_vec())),
        Item::Penalty { ahead, .. } => {
            // A window only counts once there is an earlier break to go back to.
            if *ahead > 0.0 && st.best.is_some() {
                st.forbid_until = st.forbid_until.max(st.used + ahead);
            }
            Ok(Step::Continue)
        }
        Item::Lazy {
            node,
            repeat,
            absolute,
        } => {
            let made = Rc::new(mx(node, *absolute, st.page)?);
            let mut out = to_fragments(&[made]);
            if *repeat {
                out.iter_mut().for_each(|f| f.origin = Some(item.clone()));
            }
            let count = out.len();
            let lazy = frags.splice(i..i + 1, out).next().unwrap();
            st.pending = Some(Pending {
                at: i,
                lazy,
                count,
                placed_then: st.placed.len(),
            });
            Ok(Step::Rewind)
        }
        _ => match fit(st, frags, i, mx)? {
            Step::Decline => overflow::overflow(st, frags, i, mx),
            s => Ok(s),
        },
    }
}

fn fit(
    st: &mut State,
    frags: &mut [Fragment],
    i: usize,
    mx: &mut Materialize,
) -> Result<Step, String> {
    let s = match &*frags[i].item {
        Item::Column { .. } => fit_column(st, frags, i, mx)?,
        Item::Row { .. } => row::fit(st, frags, i, mx)?,
        _ => fit_leaf(st, &frags[i]),
    };
    if !matches!(s, Step::Continue) {
        return Ok(s);
    }
    st.used += height(&frags[i]);
    let forbid_next = frags.get(i + 1).is_some_and(|f| f.item.is_forbid());
    if !forbid_next && st.used >= st.forbid_until {
        st.best = Some((i + 1, st.placed.len()));
    }
    Ok(Step::Continue)
}

fn fit_leaf(st: &mut State, f: &Fragment) -> Step {
    let h = height(f);
    if st.used + h > st.height {
        return Step::Decline;
    }
    st.placed.push(Placed {
        item: f.item.clone(),
        y: st.used,
        height: h,
        first: f.first,
        last: true,
        children: None,
    });
    Step::Continue
}

/// Fills a column's children into the space left. When none of them places it declines, so
/// the column moves whole.
fn fit_column(
    st: &mut State,
    frags: &mut [Fragment],
    i: usize,
    mx: &mut Materialize,
) -> Result<Step, String> {
    let avail = st.height - st.used;
    if avail <= 0.0 {
        return Ok(Step::Decline);
    }
    let can_force = st.can_force && st.placed.is_empty();
    let above = st.content_above(frags);
    let mut children = std::mem::take(&mut frags[i].children);
    let inner = fill(&mut children, avail, st.page, can_force, above, mx);
    frags[i].children = children;
    let inner = inner?;
    let (item, first) = (frags[i].item.clone(), frags[i].first);
    let broke = !inner.remaining.is_empty();
    if broke && inner.placed.is_empty() {
        if inner.remaining.len() == frags[i].children.len() {
            return Ok(Step::Decline);
        }
        // A forced break came before anything placed: the page ends here.
        let mut rest = vec![Fragment {
            item,
            first,
            children: inner.remaining,
            origin: None,
        }];
        rest.extend_from_slice(&frags[i + 1..]);
        return Ok(st.done(rest));
    }
    let repeats = if broke {
        overflow::repeats(&frags[i], &inner, above)
    } else {
        Vec::new()
    };
    st.placed.push(Placed {
        item: item.clone(),
        y: st.used,
        height: if broke { avail } else { reach(&inner.placed) },
        first,
        last: !broke,
        children: Some(inner.placed),
    });
    if !broke {
        return Ok(Step::Continue);
    }
    let mut children = repeats;
    children.extend(inner.remaining);
    let mut rest = vec![Fragment {
        item,
        first: false,
        children,
        origin: None,
    }];
    rest.extend_from_slice(&frags[i + 1..]);
    Ok(st.done(rest))
}
