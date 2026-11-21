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
//! - Functions other than `render` (`hyphenationCallback`, `onRender`, ...) are not sent.

pub mod node;

pub use node::{Kind, Node};

use serde::Deserialize;
use serde_json::{Map, Value};

/// A node as it arrives in the JSON.
#[derive(Debug, Deserialize)]
pub struct RawNode {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub props: Map<String, Value>,
    #[serde(default)]
    pub style: Value,
    #[serde(default)]
    pub render: Option<u32>,
    #[serde(default)]
    pub children: Vec<RawNode>,
    /// The text of a `TEXT_INSTANCE`.
    #[serde(default)]
    pub value: Option<String>,
}

/// What a `render` callback returns.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    #[serde(default)]
    pub children: Vec<RawNode>,
    #[serde(default)]
    pub reads_total: bool,
}

/// Parses a document. The root must be a `DOCUMENT` of version 1.
pub fn parse(json: &str) -> Result<RawNode, String> {
    #[derive(Deserialize)]
    struct Versioned {
        v: u32,
        #[serde(flatten)]
        root: RawNode,
    }
    let doc: Versioned = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if doc.v != 1 {
        return Err(format!("unsupported document version {}", doc.v));
    }
    if doc.root.kind != "DOCUMENT" {
        return Err(format!("the root must be a DOCUMENT, not {}", doc.root.kind));
    }
    Ok(doc.root)
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
        let page = &doc.children[0];
        assert!(page.style.is_array());
        assert_eq!(page.children[0].render, Some(3));
        assert_eq!(page.children[0].children[0].value.as_deref(), Some("hi"));
    }

    #[test]
    fn rejects_other_versions() {
        assert!(parse(r#"{"v":2,"type":"DOCUMENT"}"#).is_err());
        assert!(parse(r#"{"v":1,"type":"PAGE"}"#).is_err());
    }
}
