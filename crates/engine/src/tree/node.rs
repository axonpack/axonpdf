//! The working tree: each node with its resolved style and, once laid out, its box.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use serde_json::{Map, Value, json};

use super::{Json, RawNode, extras, truthy};
use crate::style::{self, Container, Style};
use crate::text::Line;
use crate::text::exclusion::Exclusion;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Page,
    View,
    Text,
    Link,
    Image,
    ImageBackground,
    Note,
    Canvas,
    Svg,
    TextInstance,
    Other(String),
}

impl Kind {
    fn from(s: &str) -> Kind {
        match s {
            "PAGE" => Kind::Page,
            "VIEW" => Kind::View,
            "TEXT" => Kind::Text,
            "LINK" => Kind::Link,
            "IMAGE" => Kind::Image,
            "IMAGE_BACKGROUND" => Kind::ImageBackground,
            "NOTE" => Kind::Note,
            "CANVAS" => Kind::Canvas,
            "SVG" => Kind::Svg,
            "TEXT_INSTANCE" => Kind::TextInstance,
            other => Kind::Other(other.to_string()),
        }
    }

    /// The type as the tree names it (`VIEW`), as warnings print it.
    pub fn name(&self) -> &str {
        match self {
            Kind::Page => "PAGE",
            Kind::View => "VIEW",
            Kind::Text => "TEXT",
            Kind::Link => "LINK",
            Kind::Image => "IMAGE",
            Kind::ImageBackground => "IMAGE_BACKGROUND",
            Kind::Note => "NOTE",
            Kind::Canvas => "CANVAS",
            Kind::Svg => "SVG",
            Kind::TextInstance => "TEXT_INSTANCE",
            Kind::Other(other) => other,
        }
    }
}

/// A node's box after layout, relative to its parent's border box. Edges are top, right,
/// bottom, left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub margin: [f64; 4],
    pub padding: [f64; 4],
    pub border: [f64; 4],
}

/// A text node's lines, kept from the first width layout asks for.
#[derive(Clone, Debug)]
pub struct TextBox {
    pub lines: Option<Vec<Line>>,
    /// Shift that undoes alignment when the text's box shrank to its lines.
    pub align_offset: f64,
    /// The floats the lines flow around, in the text's content box. `None` for a piece of a
    /// text cut by a page break: its lines stay as they were cut.
    pub wrapped: Option<Vec<Exclusion>>,
}

impl Default for TextBox {
    fn default() -> Self {
        TextBox {
            lines: None,
            align_offset: 0.0,
            wrapped: Some(Vec::new()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub props: Map<String, Value>,
    /// Shared between nodes that resolve to the same style; copied when a cut changes it.
    pub style: Rc<Style>,
    pub render: Option<u32>,
    pub paint: Option<u32>,
    pub children: Vec<Node>,
    /// The text of a `TEXT_INSTANCE`.
    pub value: String,
    pub layout: Layout,
    /// Set by layout on `Text` nodes.
    pub text: Option<Rc<RefCell<TextBox>>>,
    /// The node's Yoga node in the tree that last laid it out.
    pub yoga: Option<usize>,
    /// Changed since that layout (style, children or lines), so Yoga must hear of it.
    pub dirty: bool,
    /// It or something under it has a `render` function.
    pub dynamic: bool,
    /// How many children the kept layout holds; the rest wait below it (0: all).
    pub window: usize,
    /// Children not built yet, after `children`: a long list is built as the cut nears it.
    pub pending: VecDeque<Json>,
    /// The child built last asked for a break after it.
    pub break_next: bool,
    /// Added to the top margin so `clear` puts the node below the floats before it.
    pub clearance: f64,
}

impl Node {
    /// Builds a node and its children, resolving styles against the page and inheriting from
    /// `parent` (its style, and whether it is a text node).
    pub fn build(raw: RawNode, c: &Container, parent: Option<(&Rc<Style>, bool)>) -> Node {
        let raw = substitute_link(extras::translate(raw));
        let kind = Kind::from(&raw.kind);
        let raw_style = if kind == Kind::Link {
            // Links are blue and underlined unless styled otherwise.
            json!([{"color": "blue", "textDecoration": "underline"}, raw.style])
        } else if kind == Kind::Svg {
            // An Svg's width and height props size its box unless its style says otherwise.
            let sized: serde_json::Map<String, Value> = ["width", "height"]
                .iter()
                .filter_map(|k| {
                    Some((
                        k.to_string(),
                        raw.props.get(*k).filter(|v| !v.is_null())?.clone(),
                    ))
                })
                .collect();
            json!([sized, raw.style])
        } else {
            raw.style
        };
        let style = shared_style(c, raw_style, parent);
        let mut node = Node {
            kind,
            props: raw.props,
            style,
            render: raw.render,
            paint: raw.paint,
            children: Vec::new(),
            value: raw.value.unwrap_or_default(),
            layout: Layout::default(),
            text: None,
            yoga: None,
            dirty: false,
            dynamic: false,
            window: 0,
            pending: VecDeque::new(),
            break_next: false,
            clearance: 0.0,
        };
        node.set_children(raw.children, c);
        node
    }

    /// Replaces the children with `raw` ones, styled under this node. Past the first
    /// [`LAZY`] children of a container, plain ones wait as JSON until [`Node::build_pending`].
    pub fn set_children(&mut self, raw: Vec<Json>, c: &Container) {
        self.children.clear();
        self.pending = raw.into();
        let container = !matches!(self.kind, Kind::Text | Kind::Svg | Kind::Note);
        let lazy = container
            && self.pending.len() > LAZY
            && self
                .pending
                .iter()
                .all(|r| !r.get().contains("\"breakAfter\""))
            && self.pending.iter().skip(LAZY).all(|r| plain(r.get()));
        self.build_pending(c, if lazy { LAZY } else { usize::MAX });
    }

    /// Builds up to `max` waiting children; returns how many.
    pub fn build_pending(&mut self, c: &Container, max: usize) -> usize {
        // Svg children keep their own styles.
        let inherit = self.kind != Kind::Svg;
        let is_text = self.kind == Kind::Text;
        let mut built = 0;
        while built < max
            && let Some(json) = self.pending.pop_front()
        {
            let r = RawNode::parse(&json);
            let after = extras::breaks_after(&r);
            let mut n = Node::build(r, c, inherit.then_some((&self.style, is_text)));
            if std::mem::replace(&mut self.break_next, after) {
                n.props.insert("break".into(), true.into());
            }
            extras::collapse(self, &mut n);
            self.children.push(n);
            built += 1;
        }
        self.dynamic = self.render.is_some()
            || self.has(extras::COUNTER)
            || self.children.iter().any(|c| c.dynamic);
        built
    }

    /// Adds `raw` children after the ones there, built now.
    pub fn append_children(
        &mut self,
        raw: impl IntoIterator<Item = impl Into<Json>>,
        c: &Container,
    ) {
        self.pending.extend(raw.into_iter().map(Into::into));
        self.build_pending(c, usize::MAX);
    }

    pub fn has(&self, prop: &str) -> bool {
        self.props.contains_key(prop)
    }

    /// Whether a prop is set to something true, the way a JS condition reads it.
    pub fn is(&self, prop: &str) -> bool {
        self.props.get(prop).is_some_and(truthy)
    }

    pub fn number(&self, prop: &str) -> Option<f64> {
        self.props.get(prop).and_then(Value::as_f64)
    }

    /// `fixed={true}` (only `true` counts).
    pub fn is_fixed(&self) -> bool {
        self.props.get("fixed") == Some(&Value::Bool(true))
    }

    /// Whether the node may split across pages. Images, SVGs, notes and canvases never do.
    pub fn can_wrap(&self) -> bool {
        if matches!(
            self.kind,
            Kind::Svg | Kind::Note | Kind::Image | Kind::Canvas
        ) {
            return false;
        }
        self.props.get("wrap").is_none_or(truthy)
    }
}

/// Children past this many wait as JSON when they are plain.
pub const LAZY: usize = 64;

/// A child that can wait as JSON: nothing in it repeats on every page, renders itself, counts
/// pages, is placed absolutely, or marks the outline. Read from the text, so a match in a
/// string only makes the node build sooner.
fn plain(json: &str) -> bool {
    ![
        "\"render\"",
        "\"fixed\"",
        "\"bookmark\"",
        "PAGE_NUMBER",
        "PAGE_COUNT",
        "absolute",
    ]
    .iter()
    .any(|w| json.contains(w))
}

/// A link with a `render` function draws as text. A link holding only text is given a text node
/// to hold it, so the text is laid out and styled under the link.
fn substitute_link(mut raw: RawNode) -> RawNode {
    if raw.kind != "LINK" {
        return raw;
    }
    if raw.render.is_some() {
        raw.kind = "TEXT".into();
        return raw;
    }
    #[derive(serde::Deserialize)]
    struct Kind0 {
        #[serde(rename = "type")]
        kind: String,
    }
    let kinds: Vec<String> = raw
        .children
        .iter()
        .map(|c| serde_json::from_str::<Kind0>(c.get()).map_or(String::new(), |k| k.kind))
        .collect();
    let all = |want: &[&str]| kinds.iter().all(|k| want.contains(&k.as_str()));
    let wrap = all(&["TEXT_INSTANCE"]) || (!all(&["TEXT"]) && all(&["TEXT", "TEXT_INSTANCE"]));
    if wrap {
        let children = std::mem::take(&mut raw.children);
        raw.children = vec![
            RawNode {
                kind: "TEXT".into(),
                props: Map::new(),
                style: Value::Null,
                render: None,
                paint: None,
                children,
                value: None,
            }
            .boxed(),
        ];
    }
    raw
}

/// The raw style, the inherited style's address, and whether it was inherited from a text.
type StyleKey = (String, usize, bool);
/// The inherited style, kept alive, and the style resolved.
type Shared = (Option<Rc<Style>>, Rc<Style>);

thread_local! {
    /// Resolved styles by what they were resolved from: the raw style as written and the style
    /// inherited from (kept alive here, so its address stays its own). Rows of a long table
    /// resolve to the same few styles.
    static STYLES: RefCell<HashMap<StyleKey, Shared>> =
        RefCell::new(HashMap::new());
}

/// Forgets the shared styles; a new page may resolve units differently.
pub fn clear_styles() {
    STYLES.with(|s| s.borrow_mut().clear());
}

/// Drops the entries whose parent style nothing outside the cache holds: no node can ask for
/// them again. Repeated, since a dropped entry can leave its own style held by the cache alone.
fn forget_dead(m: &mut HashMap<StyleKey, Shared>) {
    loop {
        let mut held: HashMap<*const Style, usize> = HashMap::new();
        for (p, s) in m.values() {
            *held.entry(Rc::as_ptr(s)).or_default() += 1;
            if let Some(p) = p {
                *held.entry(Rc::as_ptr(p)).or_default() += 1;
            }
        }
        let before = m.len();
        m.retain(|_, (p, _)| {
            p.as_ref()
                .is_none_or(|p| Rc::strong_count(p) > held[&Rc::as_ptr(p)])
        });
        if m.len() == before {
            return;
        }
    }
}

fn shared_style(c: &Container, raw: Value, parent: Option<(&Rc<Style>, bool)>) -> Rc<Style> {
    let key = (
        raw.to_string(),
        parent.map_or(0, |(p, _)| Rc::as_ptr(p) as usize),
        parent.is_some_and(|(_, t)| t),
    );
    if let Some((_, s)) = STYLES.with(|m| m.borrow().get(&key).cloned()) {
        return s;
    }
    let mut style = style::resolve(c, &raw);
    if let Some((p, is_text)) = parent {
        style::inherit(p, is_text, &mut style);
    }
    let style = Rc::new(style);
    STYLES.with(|m| {
        let mut m = m.borrow_mut();
        // A page cut gives the rest of a split box a style of its own, so rows built under it
        // add entries on every page. Each time the cache doubles, drop those whose parent only
        // the cache still holds, so it stays the size of what is alive.
        if m.len() >= 64 && m.len().is_power_of_two() {
            forget_dead(&mut m);
        }
        m.insert(key, (parent.map(|(p, _)| p.clone()), style.clone()))
    });
    style
}
