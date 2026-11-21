//! Images: decoded once when added, kept compressed, and written into the PDF once.

mod decode;
mod jpeg;

pub use crate::display::ImageId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageError {
    /// Not PNG, JPEG, GIF or WebP.
    UnknownFormat,
    Invalid(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageError::UnknownFormat => write!(f, "Unknown image format"),
            ImageError::Invalid(why) => write!(f, "Invalid image: {why}"),
        }
    }
}

impl std::error::Error for ImageError {}

pub struct Image {
    /// Pixel size as stored.
    pub width: u32,
    pub height: u32,
    /// EXIF orientation 1 to 8 (JPEG only; 1 otherwise).
    pub orientation: u8,
    pub data: ImageData,
}

pub enum ImageData {
    /// The JPEG file as is (DCTDecode); `components` is 1 (gray), 3 (RGB) or 4 (CMYK).
    Jpeg { bytes: Vec<u8>, components: u8 },
    /// 8-bit samples, zlib-compressed; the alpha channel goes into a soft mask.
    Raw {
        gray: bool,
        color: Vec<u8>,
        alpha: Option<Vec<u8>>,
    },
    /// Already written; the bytes were freed.
    Released,
}

impl Image {
    /// Size as drawn: width and height swap for the EXIF orientations that turn the image.
    pub fn size(&self) -> (u32, u32) {
        if self.orientation > 4 {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        }
    }
}

#[derive(Default)]
pub struct ImageStore {
    images: Vec<Image>,
}

impl ImageStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decodes a PNG, JPEG, GIF (first frame) or WebP file. Add each source once and reuse its
    /// id on every page.
    pub fn add(&mut self, bytes: &[u8]) -> Result<ImageId, ImageError> {
        let image = decode::decode(bytes)?;
        self.images.push(image);
        Ok(ImageId(self.images.len() as u32 - 1))
    }

    pub fn get(&self, id: ImageId) -> &Image {
        &self.images[id.0 as usize]
    }

    /// Size as drawn, in pixels (see [`Image::size`]).
    pub fn size(&self, id: ImageId) -> (u32, u32) {
        self.get(id).size()
    }
}

/// Maps the image's unit square onto `(x, y, w, h)` (top-left origin, y down), turned and
/// mirrored for its EXIF orientation.
pub fn placement(orientation: u8, x: f32, y: f32, w: f32, h: f32) -> [f32; 6] {
    match orientation {
        2 => [-w, 0.0, 0.0, -h, x + w, y + h],
        3 => [-w, 0.0, 0.0, h, x + w, y],
        4 => [w, 0.0, 0.0, h, x, y],
        5 => [0.0, h, -w, 0.0, x + w, y],
        6 => [0.0, h, w, 0.0, x, y],
        7 => [0.0, -h, w, 0.0, x, y + h],
        8 => [0.0, -h, -w, 0.0, x + w, y + h],
        _ => [w, 0.0, 0.0, -h, x, y + h],
    }
}
