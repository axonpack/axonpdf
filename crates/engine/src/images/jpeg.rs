//! Reads what a JPEG needs to be passed through: size, components and EXIF orientation.

use super::ImageError;

pub struct JpegInfo {
    pub width: u32,
    pub height: u32,
    pub components: u8,
    pub orientation: u8,
}

pub fn read(bytes: &[u8]) -> Result<JpegInfo, ImageError> {
    let bad = |why: &str| ImageError::Invalid(why.into());
    let mut i = 2;
    let mut orientation = 1;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xFF {
            return Err(bad("JPEG marker expected"));
        }
        let marker = bytes[i + 1];
        if marker == 0xFF {
            i += 1; // fill byte
            continue;
        }
        let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        let body = bytes
            .get(i + 4..i + 2 + len)
            .ok_or_else(|| bad("truncated JPEG"))?;
        match marker {
            0xE1 if body.starts_with(b"Exif\0\0") => {
                orientation = exif_orientation(&body[6..]).unwrap_or(1);
            }
            // SOF0 to SOF15, without DHT (C4), JPG (C8) and DAC (CC).
            0xC0..=0xCF if !matches!(marker, 0xC4 | 0xC8 | 0xCC) => {
                let [_, h1, h0, w1, w0, components, ..] = *body else {
                    return Err(bad("short JPEG frame header"));
                };
                return Ok(JpegInfo {
                    width: u16::from_be_bytes([w1, w0]) as u32,
                    height: u16::from_be_bytes([h1, h0]) as u32,
                    components,
                    orientation,
                });
            }
            _ => {}
        }
        i += 2 + len;
    }
    Err(bad("no JPEG frame header"))
}

/// Tag 0x0112 of the first IFD of a TIFF block.
fn exif_orientation(tiff: &[u8]) -> Option<u8> {
    let le = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |at: usize| {
        let b = [*tiff.get(at)?, *tiff.get(at + 1)?];
        Some(if le {
            u16::from_le_bytes(b)
        } else {
            u16::from_be_bytes(b)
        })
    };
    let u32_at = |at: usize| {
        let b: [u8; 4] = tiff.get(at..at + 4)?.try_into().ok()?;
        Some(if le {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        })
    };
    let ifd = u32_at(4)? as usize;
    for n in 0..u16_at(ifd)? as usize {
        let entry = ifd + 2 + n * 12;
        if u16_at(entry)? == 0x0112 {
            let value = u16_at(entry + 8)?;
            return (1..=8).contains(&value).then_some(value as u8);
        }
    }
    None
}
