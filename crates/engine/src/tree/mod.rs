//! The document tree: the one contract between the component packages and the engine.
//!
//! Every adapter (React, Vue, Flutter, plain JS) builds this JSON and hands it to
//! [`crate::Engine::render`]. The engine never sees anything else from the app.
//!
//! ```json
//! {
//!   "v": 1,
//!   "type": "DOCUMENT",
//!   "props": { "title": "...", "author": "...", "subject": "...", "keywords": "...",
//!              "creator": "...", "producer": "...", "language": "...",
//!              "pageMode": "...", "pageLayout": "..." },
//!   "style": {},
//!   "children": [ <PAGE node>, ... ]
//! }
//! ```
//!
//! A node:
//!
//! ```json
//! {
//!   "type": "PAGE" | "VIEW" | "TEXT" | "LINK" | "IMAGE" | "NOTE" | ...,
//!   "props": { ... },
//!   "style": <the style exactly as written>,
//!   "render": <callback id>,
//!   "children": [ <node> | { "type": "TEXT_INSTANCE", "value": "text" } ]
//! }
//! ```
//!
//! - `type` is the primitive's name in upper case, as the components declare it.
//! - `props` holds every prop that is neither `style`, `children` nor a function, as written:
//!   `size`, `orientation`, `wrap`, `fixed`, `break`, `minPresenceAhead`, `id`, `bookmark`,
//!   `debug`, `orphans`, `widows`, `dpi`, `href`, `hyphenationPenalty`, and so on. A prop left
//!   `undefined` is left out; `false` and `null` are kept, since the engine tells them apart
//!   from a missing prop where the components do.
//! - An image source travels as `"src": { "asset": <id> }`, the id given to
//!   [`crate::Engine::add_image`] for its bytes. A link's `href` (or `src`) stays a string.
//! - `style` is passed through untouched: an object, or an array of objects, nested arrays,
//!   `null` and `false` entries included. Values keep their units as strings (`"10 20"`,
//!   `"1pt solid red"`, `"50%"`, `"2in"`), and media queries stay as `"@media ..."` keys.
//!   The engine resolves all of it.
//! - `render` is present when the node has a `render` prop. The function stays with the
//!   adapter under that id, and the engine calls it back while it paginates (see
//!   [`crate::RenderArgs`]). The callback answers `{"children": [<node>...], "readsTotal": bool}`,
//!   the children in this same shape.
//! - `layout` (on `PAGE`): the id of the page's `layout` template function. The engine calls it
//!   through the same callback as `render`, with the same [`crate::RenderArgs`] (`pageNumber`,
//!   `totalPages`, `subPageNumber`, `subPageTotalPages`), and the adapter calls the function with
//!   those plus `children`, a placeholder that renders as one `{"type": "SLOT"}` node. The answer
//!   is `{"children": [<node>...], "readsTotal": bool}` with exactly one `SLOT` somewhere in it;
//!   the page's content flows where it is, the rest is drawn on every page with its room kept.
//!   It is called once for page 1 with only `pageNumber` (to lay the content out at the slot's
//!   width), then once per page cut. Any page with `layout`, or with the
//!   `"experimentalPagination": true` prop, switches the whole document to the newer pagination
//!   engine ([`crate::paginate::next`]).
//! - `paint` (on `CANVAS`): the id of the `paint` function. The engine calls it with the
//!   canvas's size (NaN, meaning `undefined`, when it only measures) and gets the painter calls
//!   back as a JSON list (see [`crate::Engine::set_paint_callback`]).
//! - `hyphenationCallback` (a `TEXT` prop, and a `DOCUMENT` prop for the one registered with the
//!   font store): the id of a function the engine calls per word
//!   ([`crate::Engine::set_hyphenation_callback`]).
//! - `emoji` (a `DOCUMENT` prop): `{"<emoji>": {"asset": <id>}}`, the image of each emoji the
//!   text holds, fetched from the registered emoji source. Emoji without an image stay text.
//! - Functions other than `render` (`hyphenationCallback`, `onRender`, ...) are not sent.

pub mod extras;
pub mod node;

pub use node::{Kind, Node};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::{Map, Value};

/// A node as it arrives in the JSON. Its children stay JSON until each is built.
#[derive(Debug, Deserialize, Serialize)]
pub struct RawNode {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub props: Map<String, Value>,
    #[serde(default)]
    pub style: Value,
    #[serde(default)]
    pub render: Option<u32>,
    /// A `CANVAS`'s `paint` function, by id.
    #[serde(default)]
    pub paint: Option<u32>,
    #[serde(default)]
    pub children: Vec<Box<RawValue>>,
    /// The text of a `TEXT_INSTANCE`.
    #[serde(default)]
    pub value: Option<String>,
}

impl RawNode {
    /// Reads one node. A node the engine cannot read is reported and left empty.
    pub fn parse(json: &RawValue) -> RawNode {
        serde_json::from_str(json.get()).unwrap_or_else(|e| {
            eprintln!("skipping a node the engine cannot read: {e}");
            serde_json::from_str(r#"{"type":"VIEW"}"#).unwrap()
        })
    }

    /// Back to JSON, to sit among other nodes' children.
    pub fn boxed(&self) -> Box<RawValue> {
        serde_json::value::to_raw_value(self).expect("a node is always JSON")
    }

    /// A `TEXT_INSTANCE`.
    pub fn text(value: String) -> RawNode {
        RawNode {
            kind: "TEXT_INSTANCE".into(),
            props: Map::new(),
            style: Value::Null,
            render: None,
            paint: None,
            children: Vec::new(),
            value: Some(value),
        }
    }
}

/// What a `render` callback returns.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    #[serde(default)]
    pub children: Vec<Box<RawValue>>,
    #[serde(default)]
    pub reads_total: bool,
}

/// A document, its pages left as JSON until each is reached, so only one page is ever held as
/// a tree.
#[derive(Debug, Deserialize)]
pub struct Doc<'a> {
    v: u32,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    pub props: Map<String, Value>,
    #[serde(borrow, default)]
    children: Vec<&'a RawValue>,
}

impl Doc<'_> {
    /// The `PAGE` children, parsed one at a time.
    pub fn pages(&self) -> impl Iterator<Item = Result<RawNode, String>> + '_ {
        self.children
            .iter()
            .map(|c| serde_json::from_str::<RawNode>(c.get()).map_err(|e| e.to_string()))
            .filter(|r| r.as_ref().map_or(true, |n| n.kind == "PAGE"))
    }

    /// The pages' JSON, unread.
    pub fn page_json(&self) -> impl Iterator<Item = &str> + '_ {
        self.children.iter().map(|c| c.get())
    }
}

/// Every number given to `key` anywhere in a JSON text. Keys are the only quoted words a
/// colon follows, so text values cannot match.
pub fn numbers_at(json: &str, key: &str) -> Vec<u32> {
    let needle = format!("\"{key}\"");
    let mut out = Vec::new();
    for (i, _) in json.match_indices(&needle) {
        let rest = json[i + needle.len()..].trim_start();
        let Some(rest) = rest.strip_prefix(':') else {
            continue;
        };
        let digits: String = rest
            .trim_start()
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(n) = digits.parse() {
            out.push(n);
        }
    }
    out
}

/// Whether a node of this `type` is anywhere in a JSON text.
pub fn has_type(json: &str, kind: &str) -> bool {
    json.match_indices("\"type\"").any(|(i, _)| {
        let rest = json[i + 6..].trim_start();
        rest.strip_prefix(':')
            .is_some_and(|r| r.trim_start().starts_with(&format!("\"{kind}\"")))
    })
}

/// Parses a document. The root must be a `DOCUMENT` of version 1.
pub fn parse(json: &str) -> Result<Doc<'_>, String> {
    let doc: Doc = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if doc.v != 1 {
        return Err(format!("unsupported document version {}", doc.v));
    }
    if doc.kind != "DOCUMENT" {
        return Err(format!("the root must be a DOCUMENT, not {}", doc.kind));
    }
    Ok(doc)
}

/// Whether a JSON value counts as true the way a JS condition reads it.
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_document() {
        let doc = parse(
            r#"{"v":1,"type":"DOCUMENT","props":{"title":"T"},"children":[
                {"type":"PAGE","props":{"size":"A4"},"style":[{"padding":10},null,false],
                 "children":[{"type":"TEXT","render":3,"children":[
                    {"type":"TEXT_INSTANCE","value":"hi"}]}]}]}"#,
        )
        .unwrap();
        assert_eq!(doc.props["title"], "T");
        let page = doc.pages().next().unwrap().unwrap();
        assert!(page.style.is_array());
        let text = RawNode::parse(&page.children[0]);
        assert_eq!(text.render, Some(3));
        assert_eq!(
            RawNode::parse(&text.children[0]).value.as_deref(),
            Some("hi")
        );
        let json = doc.page_json().next().unwrap();
        assert_eq!(numbers_at(json, "render"), [3]);
        assert!(has_type(json, "TEXT_INSTANCE") && !has_type(json, "ROWS"));
    }

    #[test]
    fn rejects_other_versions() {
        assert!(parse(r#"{"v":2,"type":"DOCUMENT"}"#).is_err());
        assert!(parse(r#"{"v":1,"type":"PAGE"}"#).is_err());
    }
}
