//! Image XObjects, written the first time a page draws them; their bytes are freed after.

use pdf_writer::{Filter, Finish, Ref};

use super::Writer;
use super::emit::Out;
use crate::display::ImageId;
use crate::images::{ImageData, ImageStore};

impl Writer {
    pub(super) fn image_ref(
        &mut self,
        id: ImageId,
        images: &mut ImageStore,
        out: Out,
    ) -> (String, Ref) {
        let name = format!("Im{}", id.0);
        if let Some(r) = self.images.get(&id) {
            return (name, *r);
        }
        let r = self.emit.alloc();
        let (width, height) = (images.get(id).width as i32, images.get(id).height as i32);
        match images.release(id) {
            ImageData::Jpeg { bytes, components } => {
                let data = self.seal(r, bytes);
                self.emit.write(out, |c| {
                    let mut x = c.image_xobject(r, &data);
                    x.width(width).height(height).bits_per_component(8);
                    x.filter(Filter::DctDecode);
                    match components {
                        1 => x.color_space().device_gray(),
                        4 => {
                            x.color_space().device_cmyk();
                            // Adobe's CMYK JPEGs store inverted samples.
                            x.decode([1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
                        }
                        _ => x.color_space().device_rgb(),
                    }
                    x.finish();
                });
            }
            ImageData::Raw { gray, color, alpha } => {
                let mask = alpha.map(|a| {
                    let m = self.emit.alloc();
                    let data = self.seal(m, a);
                    self.emit.write(out, |c| {
                        let mut x = c.image_xobject(m, &data);
                        x.width(width).height(height).bits_per_component(8);
                        x.filter(Filter::FlateDecode);
                        x.color_space().device_gray();
                    });
                    m
                });
                let data = self.seal(r, color);
                self.emit.write(out, |c| {
                    let mut x = c.image_xobject(r, &data);
                    x.width(width).height(height).bits_per_component(8);
                    x.filter(Filter::FlateDecode);
                    if gray {
                        x.color_space().device_gray()
                    } else {
                        x.color_space().device_rgb()
                    };
                    if let Some(m) = mask {
                        x.s_mask(m);
                    }
                });
            }
            ImageData::Released => unreachable!("released images are in the written map"),
        }
        self.images.insert(id, r);
        (name, r)
    }
}
