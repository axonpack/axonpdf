//! A node's JSON left unread, as a slice of a shared text: children waiting to be built point
//! into the document's own JSON instead of holding a copy of it.

use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

#[derive(Clone, Default)]
pub struct Json {
    text: Rc<String>,
    range: Range<usize>,
}

impl Json {
    /// All of `text`.
    pub fn new(text: String) -> Json {
        let range = 0..text.len();
        Json {
            text: Rc::new(text),
            range,
        }
    }

    /// The JSON text.
    pub fn get(&self) -> &str {
        &self.text[self.range.clone()]
    }

    /// `part`, a slice of this JSON's text (as serde borrows it), sharing the text.
    pub fn part(&self, part: &str) -> Json {
        let start = (part.as_ptr() as usize).wrapping_sub(self.text.as_ptr() as usize);
        assert!(
            start <= self.text.len() && part.len() <= self.text.len() - start,
            "not a slice of this text"
        );
        Json {
            text: self.text.clone(),
            range: start..start + part.len(),
        }
    }
}

impl From<Box<RawValue>> for Json {
    fn from(raw: Box<RawValue>) -> Json {
        Json::new(Box::<str>::from(raw).into_string())
    }
}

impl fmt::Debug for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get())
    }
}

/// Read on its own (not through [`super::RawNode::parse`]), a node's JSON is copied.
impl<'de> Deserialize<'de> for Json {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Json, D::Error> {
        Box::<RawValue>::deserialize(d).map(Json::from)
    }
}

impl Serialize for Json {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        RawValue::from_string(self.get().to_owned())
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
}
