//! Links, notes, named destinations and the outline.

use std::collections::HashMap;

use pdf_writer::{Finish, Name, Null, Ref, Str};

use super::Writer;
use super::emit::Out;
use super::strings::text;
use crate::display::{Bookmark, Color, LinkTarget};

/// What the end of the document needs from the pages: a few values per entry.
#[derive(Default)]
pub struct Nav {
    dests: Vec<(String, Ref, f32, f32)>,
    bookmarks: Vec<(Bookmark, Ref, f32)>,
}

pub struct NavRefs {
    pub outline: Option<Ref>,
    pub dests: Option<Ref>,
    pub form: Option<Ref>,
    pub pdfa: Option<super::pdfa::PdfaRefs>,
}

impl Writer {
    /// A destination at `(x, y)` in top-left page coordinates.
    pub(super) fn add_dest(&mut self, name: &str, page: Ref, x: f32, y: f32, height: f32) {
        self.nav.dests.push((name.to_string(), page, x, height - y));
    }

    pub(super) fn add_bookmark(&mut self, b: &Bookmark, page: Ref, height: f32) {
        self.nav.bookmarks.push((b.clone(), page, height));
    }

    /// A link annotation over `rect` (`[x0 y0 x1 y1]` in PDF space).
    pub(super) fn link(&mut self, rect: [f32; 4], target: &LinkTarget, out: Out) -> Ref {
        let id = self.emit.alloc();
        let (kind, key, value): (&[u8], &[u8], _) = match target {
            LinkTarget::Uri(uri) => (b"URI", b"URI", self.seal(id, uri.as_bytes().to_vec())),
            LinkTarget::Dest(name) => (b"GoTo", b"D", self.seal(id, text(name))),
        };
        self.emit.write(out, |c| {
            let mut a = c.indirect(id).dict();
            annot_base(&mut a, b"Link", rect);
            a.pair(Name(b"F"), 4);
            let mut action = a.insert(Name(b"A")).dict();
            action
                .pair(Name(b"S"), Name(kind))
                .pair(Name(key), Str(&value));
        });
        id
    }

    /// A sticky note at a point (`[x y]` in PDF space). Yellow unless `color` is given.
    pub(super) fn note(
        &mut self,
        at: (f32, f32),
        contents: &str,
        color: Option<Color>,
        out: Out,
    ) -> Ref {
        let id = self.emit.alloc();
        let contents = self.seal(id, text(contents));
        let c = color.unwrap_or(Color {
            r: 243.0 / 255.0,
            g: 223.0 / 255.0,
            b: 92.0 / 255.0,
            a: 1.0,
        });
        self.emit.write(out, |chunk| {
            let mut a = chunk.indirect(id).dict();
            annot_base(&mut a, b"Text", [at.0, at.1, at.0, at.1]);
            a.pair(Name(b"Contents"), Str(&contents));
            a.pair(Name(b"Name"), Name(b"Comment"));
            a.insert(Name(b"C")).array().items([c.r, c.g, c.b]);
        });
        id
    }

    pub(super) fn write_nav(&mut self, _pages: &[Ref], out: Out) -> NavRefs {
        let dests = (!self.nav.dests.is_empty()).then(|| self.write_dests(out));
        let outline = (!self.nav.bookmarks.is_empty()).then(|| self.write_outline(out));
        NavRefs {
            outline,
            dests,
            form: None,
            pdfa: None,
        }
    }

    fn write_dests(&mut self, out: Out) -> Ref {
        let id = self.emit.alloc();
        let mut dests = std::mem::take(&mut self.nav.dests);
        dests.sort_by_key(|d| text(&d.0));
        dests.dedup_by(|a, b| a.0 == b.0);
        let names: Vec<Vec<u8>> = dests.iter().map(|d| self.seal(id, text(&d.0))).collect();
        self.emit.write(out, |c| {
            let mut dict = c.indirect(id).dict();
            let mut list = dict.insert(Name(b"Names")).array();
            for ((_, page, x, y), name) in dests.iter().zip(&names) {
                list.item(Str(name));
                list.push()
                    .array()
                    .item(*page)
                    .item(Name(b"XYZ"))
                    .item(*x)
                    .item(*y)
                    .item(Null);
            }
        });
        id
    }

    fn write_outline(&mut self, out: Out) -> Ref {
        let root = self.emit.alloc();
        let items = std::mem::take(&mut self.nav.bookmarks);
        let refs: Vec<Ref> = items.iter().map(|_| self.emit.alloc()).collect();
        let index: HashMap<u32, usize> =
            items.iter().enumerate().map(|(i, b)| (b.0.id, i)).collect();
        // Children by parent (None for top level), in document order.
        let mut kids: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
        for (i, (b, ..)) in items.iter().enumerate() {
            let parent = b
                .parent
                .and_then(|p| index.get(&p).copied())
                .filter(|&p| p < i);
            kids.entry(parent).or_default().push(i);
        }
        let top = kids.get(&None).cloned().unwrap_or_default();
        self.emit.write(out, |c| {
            let mut d = c.indirect(root).dict();
            d.pair(Name(b"Type"), Name(b"Outlines"));
            d.pair(Name(b"First"), refs[top[0]])
                .pair(Name(b"Last"), refs[*top.last().unwrap()]);
        });
        for (i, (b, page, height)) in items.iter().enumerate() {
            let parent = b
                .parent
                .and_then(|p| index.get(&p).copied())
                .filter(|&p| p < i);
            let siblings = &kids[&parent];
            let at = siblings.iter().position(|&s| s == i).unwrap();
            let title = self.seal(refs[i], text(&b.title));
            let children = kids.get(&Some(i));
            self.emit.write(out, |c| {
                let mut d = c.indirect(refs[i]).dict();
                d.pair(Name(b"Title"), Str(&title));
                d.pair(Name(b"Parent"), parent.map_or(root, |p| refs[p]));
                if at > 0 {
                    d.pair(Name(b"Prev"), refs[siblings[at - 1]]);
                }
                if let Some(&next) = siblings.get(at + 1) {
                    d.pair(Name(b"Next"), refs[next]);
                }
                if let Some(children) = children {
                    d.pair(Name(b"First"), refs[children[0]])
                        .pair(Name(b"Last"), refs[*children.last().unwrap()]);
                    if b.expanded {
                        d.pair(Name(b"Count"), children.len() as i32);
                    }
                }
                let mut dest = d.insert(Name(b"Dest")).array();
                dest.item(*page);
                if b.fit {
                    dest.item(Name(b"Fit"));
                } else {
                    dest.item(Name(b"XYZ"))
                        .item(b.left)
                        .item(height - b.top)
                        .item(b.zoom.unwrap_or(0.0));
                }
                dest.finish();
            });
        }
        root
    }
}

pub(super) fn annot_base(a: &mut pdf_writer::Dict, subtype: &[u8], rect: [f32; 4]) {
    a.pair(Name(b"Type"), Name(b"Annot"));
    a.pair(Name(b"Subtype"), Name(subtype));
    a.insert(Name(b"Rect")).array().items(rect);
    a.insert(Name(b"Border")).array().items([0, 0, 0]);
}
