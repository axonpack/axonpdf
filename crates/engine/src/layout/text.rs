//! A text node for layout: its fragments, and the measure Yoga calls.

use std::cell::RefCell;
use std::rc::Rc;

use super::yoga::MeasureMode;
use crate::display::Color;
use crate::fonts::FontStore;
use crate::text::{self, Attrs, Fragment, Options, Paragraph};
use crate::tree::node::TextBox;
use crate::tree::{Kind, Node};

/// The styled pieces of a `Text` and everything nested in it.
pub fn fragments(
    fonts: &FontStore,
    node: &Node,
    link: Option<&str>,
    level: usize,
    out: &mut Vec<Fragment>,
) -> Result<(), String> {
    let s = &node.style;
    let families = s
        .font_family
        .clone()
        .unwrap_or_else(|| vec!["Helvetica".into()]);
    let weight = s.font_weight.unwrap_or(400);
    let italic = matches!(s.font_style.as_deref(), Some("italic" | "oblique"));
    let stack = families
        .iter()
        .map(String::as_str)
        .chain(["Helvetica"])
        .map(|f| fonts.resolve(f, weight, italic).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let decoration = s.text_decoration.as_deref().unwrap_or("");
    let color = s.color.unwrap_or(Color::BLACK);
    let link = link.map(str::to_string).or_else(|| {
        node.props
            .get("src")
            .or(node.props.get("href"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    });
    let attrs = Rc::new(Attrs {
        size: s.font_size.unwrap_or(18.0),
        color,
        opacity: s.opacity,
        letter_spacing: s.letter_spacing.unwrap_or(0.0),
        line_height: s.line_height,
        underline: matches!(
            decoration,
            "underline" | "underline line-through" | "line-through underline"
        ),
        strike: matches!(
            decoration,
            "line-through" | "underline line-through" | "line-through underline"
        ),
        decoration_color: s.text_decoration_color.unwrap_or(color),
        decoration_style: s
            .text_decoration_style
            .clone()
            .unwrap_or_else(|| "solid".into()),
        link: link.clone(),
        align: s.text_align.clone().unwrap_or_else(|| {
            if s.direction.as_deref() == Some("rtl") {
                "right".into()
            } else {
                "left".into()
            }
        }),
        // The outer text's own background is drawn as a box, not per run.
        background: if level == 0 { None } else { s.background_color },
        indent: s.text_indent.unwrap_or(0.0),
        vertical_align: s.vertical_align.clone(),
    });
    for child in &node.children {
        match child.kind {
            Kind::TextInstance => out.push(Fragment {
                text: transform(&child.value, s.text_transform.as_deref()),
                attrs: attrs.clone(),
                fonts: stack.clone(),
            }),
            // ponytail: images inside text are skipped until inline attachments land.
            Kind::Image => {}
            _ => fragments(fonts, child, link.as_deref(), level + 1, out)?,
        }
    }
    Ok(())
}

fn transform(text: &str, how: Option<&str>) -> String {
    match how {
        Some("uppercase") => text.to_uppercase(),
        Some("lowercase") => text.to_lowercase(),
        Some("capitalize") => {
            let mut prev_space = true;
            text.chars()
                .map(|c| {
                    let up = prev_space && !c.is_whitespace();
                    prev_space = c.is_whitespace();
                    if up {
                        c.to_uppercase().collect::<String>()
                    } else {
                        c.to_string()
                    }
                })
                .collect()
        }
        Some("upperfirst") => {
            let mut cs = text.chars();
            cs.next()
                .map_or(String::new(), |f| f.to_uppercase().chain(cs).collect())
        }
        _ => text.to_string(),
    }
}

pub fn options(node: &Node) -> Options {
    Options {
        max_lines: node
            .style
            .max_lines
            .filter(|m| *m >= 0.0)
            .map(|m| m as usize),
        ellipsis: node.style.text_overflow.as_deref() == Some("ellipsis"),
        hyphenation_penalty: node.number("hyphenationPenalty"),
    }
}

/// Yoga's measure for a text: lines are made on the first call and kept, whatever width a
/// later call asks for.
pub fn measure(
    state: Rc<RefCell<TextBox>>,
    paragraphs: Option<Rc<Vec<Paragraph>>>,
    opts: Options,
    align: Option<String>,
) -> impl FnMut(f32, MeasureMode, f32, MeasureMode) -> (f32, f32) {
    move |w, wm, h, _| {
        let (w, h) = (w as f64, h as f64);
        let mut st = state.borrow_mut();
        let layout = |st: &mut TextBox| {
            if st.lines.is_none()
                && let Some(p) = &paragraphs
            {
                st.lines = Some(text::layout(p, w, (!h.is_nan()).then_some(h), &opts));
                return true;
            }
            false
        };
        let height = |st: &TextBox| {
            st.lines
                .as_ref()
                .map_or(-1.0, |ls| ls.iter().map(|l| l.height).sum())
        };
        let width = |st: &TextBox| {
            st.lines
                .as_ref()
                .map_or(0.0, |ls| ls.iter().map(|l| l.x_advance).fold(0.0, f64::max))
        };
        match wm {
            MeasureMode::Exactly => {
                layout(&mut st);
                (w as f32, height(&st) as f32)
            }
            MeasureMode::AtMost => {
                if layout(&mut st) {
                    let factor = match align.as_deref() {
                        Some("center") => 0.5,
                        Some("right") => 1.0,
                        _ => 0.0,
                    };
                    st.align_offset = (w - width(&st)) * factor;
                }
                (w.min(width(&st)) as f32, height(&st) as f32)
            }
            MeasureMode::Undefined => (f32::NAN, f32::NAN),
        }
    }
}
