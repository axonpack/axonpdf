//! The working tree: each node with its resolved style and, once laid out, its box.

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::{Map, Value, json};

use super::{RawNode, truthy};
use crate::style::{self, Container, Style};
use crate::text::Line;

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
#[derive(Clone, Debug, Default)]
pub struct TextBox {
    pub lines: Option<Vec<Line>>,
    /// Shift that undoes alignment when the text's box shrank to its lines.
    pub align_offset: f64,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub props: Map<String, Value>,
    pub style: Style,
    pub render: Option<u32>,
    pub paint: Option<u32>,
    pub children: Vec<Node>,
    /// The text of a `TEXT_INSTANCE`.
    pub value: String,
    pub layout: Layout,
    /// Set by layout on `Text` nodes.
    pub text: Option<Rc<RefCell<TextBox>>>,
}

impl Node {
    /// Builds a node and its children, resolving styles against the page and inheriting from
    /// `parent` (its style, and whether it is a text node).
    pub fn build(raw: RawNode, c: &Container, parent: Option<(&Style, bool)>) -> Node {
        let raw = substitute_link(raw);
        let kind = Kind::from(&raw.kind);
        let mut style = if kind == Kind::Link {
            // Links are blue and underlined unless styled otherwise.
            style::resolve(
                c,
                &json!([{"color": "blue", "textDecoration": "underline"}, raw.style]),
            )
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
            style::resolve(c, &json!([sized, raw.style]))
        } else {
            style::resolve(c, &raw.style)
        };
        if let Some((p, is_text)) = parent {
            style::inherit(p, is_text, &mut style);
        }
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
        };
        node.set_children(raw.children, c);
        node
    }

    /// Replaces the children with `raw` ones, styled under this node.
    pub fn set_children(&mut self, raw: Vec<RawNode>, c: &Container) {
        // Svg children keep their own styles.
        let inherit = self.kind != Kind::Svg;
        let is_text = self.kind == Kind::Text;
        self.children = raw
            .into_iter()
            .map(|r| Node::build(r, c, inherit.then_some((&self.style, is_text))))
            .collect();
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
    let all = |kinds: &[&str]| {
        raw.children
            .iter()
            .all(|c| kinds.contains(&c.kind.as_str()))
    };
    let wrap = all(&["TEXT_INSTANCE"]) || (!all(&["TEXT"]) && all(&["TEXT", "TEXT_INSTANCE"]));
    if wrap {
        let children = std::mem::take(&mut raw.children);
        raw.children = vec![RawNode {
            kind: "TEXT".into(),
            props: Map::new(),
            style: Value::Null,
            render: None,
            paint: None,
            children,
            value: None,
        }];
    }
    raw
}
