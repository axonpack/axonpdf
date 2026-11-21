//! SVG images: drawn once into a form XObject in their own points, then placed like any image.

use pdf_writer::{Filter, Ref};
use serde_json::Value;

use super::Writer;
use super::emit::Out;
use crate::display::{PageOut, Rect};
use crate::fonts::FontStore;
use crate::images::ImageStore;
use crate::style::Container;
use crate::tree::{Node, RawNode};

impl Writer {
    /// Writes the SVG `tree` (`width` × `height` points) as form XObject `r`, which maps it onto
    /// the unit square the way image XObjects are placed.
    pub(super) fn write_svg_image(
        &mut self,
        r: Ref,
        tree: Value,
        (width, height): (f32, f32),
        fonts: &FontStore,
        images: &mut ImageStore,
        out: Out,
    ) {
        let mut ops = Vec::new();
        if let (Ok(raw), true) = (
            serde_json::from_value::<RawNode>(tree),
            width > 0.0 && height > 0.0,
        ) {
            let mut svg = Node::build(raw, &Container::default(), None);
            svg.layout.width = width as f64;
            svg.layout.height = height as f64;
            // Fitted like an image: the viewBox, centred, scaled to fit.
            svg.props
                .insert("preserveAspectRatio".into(), "xMidYMid meet".into());
            let (lookup, paint) = (|_: u32| None, |_: u32, _: f64, _: f64| None);
            let cx = crate::draw::Ctx {
                fonts,
                images: &lookup,
                paint: &paint,
                forms: Default::default(),
                outline: &Default::default(),
            };
            crate::draw::svg::draw(&mut ops, &svg, 0.0, 0.0, [0.0; 4], &cx);
        }
        let (w, h) = (width.max(1.0), height.max(1.0));
        let parts = self.content(
            &PageOut {
                width: w,
                height: h,
                ops,
            },
            self.catalog,
            fonts,
            images,
            out,
        );
        let data = self.seal(
            r,
            miniz_oxide::deflate::compress_to_vec_zlib(&parts.content, 6),
        );
        self.svg_sizes.insert(r, (w, h));
        self.emit.write(out, |c| {
            let mut x = c.form_xobject(r, &data);
            x.bbox(pdf_writer::Rect::new(0.0, 0.0, w, h));
            x.matrix([1.0 / w, 0.0, 0.0, 1.0 / h, 0.0, 0.0]);
            x.filter(Filter::FlateDecode);
            // A group: the image's opacity applies to the drawing as a whole, and inside it
            // alpha starts at 1 as the content assumes.
            x.group().transparency();
            parts.resources.write(x.resources());
        });
    }
}

/// The largest box of `w` × `h` proportions centred in `r`.
pub fn meet(r: Rect, (w, h): (f32, f32)) -> Rect {
    let s = (r.w / w).min(r.h / h);
    let (fw, fh) = (w * s, h * s);
    Rect {
        x: r.x + (r.w - fw) / 2.0,
        y: r.y + (r.h - fh) / 2.0,
        w: fw,
        h: fh,
    }
}
