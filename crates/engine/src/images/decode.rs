//! Turns image files into what the PDF stores: JPEG as is, everything else as 8-bit samples.

use std::io::Cursor;

use miniz_oxide::deflate::compress_to_vec_zlib;

use super::{Image, ImageData, ImageError, jpeg};

pub fn decode(bytes: &[u8]) -> Result<Image, ImageError> {
    let invalid = |e: &dyn std::fmt::Display| ImageError::Invalid(e.to_string());
    if bytes.starts_with(&[0xFF, 0xD8]) {
        let info = jpeg::read(bytes)?;
        if !matches!(info.components, 1 | 3 | 4) {
            return Err(ImageError::Invalid(
                "unsupported JPEG color components".into(),
            ));
        }
        let data = ImageData::Jpeg {
            bytes: bytes.to_vec(),
            components: info.components,
        };
        return Ok(Image {
            width: info.width,
            height: info.height,
            orientation: info.orientation,
            data,
        });
    }
    if bytes.starts_with(b"\x89PNG") {
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().map_err(|e| invalid(&e))?;
        let mut buf = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or_else(|| invalid(&"image too large"))?
        ];
        let frame = reader.next_frame(&mut buf).map_err(|e| invalid(&e))?;
        buf.truncate(frame.buffer_size());
        let channels = frame.color_type.samples();
        return Ok(raw(frame.width, frame.height, &buf, channels));
    }
    if bytes.starts_with(b"GIF8") {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options
            .read_info(Cursor::new(bytes))
            .map_err(|e| invalid(&e))?;
        let frame = decoder
            .read_next_frame()
            .map_err(|e| invalid(&e))?
            .ok_or_else(|| invalid(&"no frame"))?;
        return Ok(raw(
            frame.width as u32,
            frame.height as u32,
            &frame.buffer,
            4,
        ));
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        let mut decoder =
            image_webp::WebPDecoder::new(Cursor::new(bytes)).map_err(|e| invalid(&e))?;
        let (width, height) = decoder.dimensions();
        let mut buf = vec![
            0;
            decoder
                .output_buffer_size()
                .ok_or_else(|| invalid(&"image too large"))?
        ];
        decoder.read_image(&mut buf).map_err(|e| invalid(&e))?;
        return Ok(raw(
            width,
            height,
            &buf,
            if decoder.has_alpha() { 4 } else { 3 },
        ));
    }
    Err(ImageError::UnknownFormat)
}

/// Splits interleaved 8-bit samples (gray, gray+alpha, RGB or RGBA) into color and alpha and
/// compresses both. An alpha channel that is fully opaque is dropped.
fn raw(width: u32, height: u32, samples: &[u8], channels: usize) -> Image {
    let has_alpha = channels == 2 || channels == 4;
    let color_channels = if has_alpha { channels - 1 } else { channels };
    let (color, alpha) = if has_alpha {
        let mut color = Vec::with_capacity(samples.len() / channels * color_channels);
        let mut alpha = Vec::with_capacity(samples.len() / channels);
        for px in samples.chunks_exact(channels) {
            color.extend_from_slice(&px[..color_channels]);
            alpha.push(px[color_channels]);
        }
        let opaque = alpha.iter().all(|&a| a == 255);
        (
            compress_to_vec_zlib(&color, 6),
            (!opaque).then(|| compress_to_vec_zlib(&alpha, 6)),
        )
    } else {
        (compress_to_vec_zlib(samples, 6), None)
    };
    let data = ImageData::Raw {
        gray: color_channels == 1,
        color,
        alpha,
    };
    Image {
        width,
        height,
        orientation: 1,
        data,
    }
}
