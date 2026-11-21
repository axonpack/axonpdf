use axonpdf::images::{ImageData, ImageStore, placement};

fn png(color: png::ColorType, w: u32, h: u32, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(color);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(data).unwrap();
    out
}

#[test]
fn png_alpha_goes_to_a_mask_only_when_used() {
    let mut store = ImageStore::new();
    let see_through = store
        .add(&png(
            png::ColorType::Rgba,
            1,
            2,
            &[255, 0, 0, 128, 0, 255, 0, 255],
        ))
        .unwrap();
    let opaque = store
        .add(&png(png::ColorType::Rgba, 1, 1, &[1, 2, 3, 255]))
        .unwrap();
    let gray = store
        .add(&png(png::ColorType::Grayscale, 2, 1, &[0, 255]))
        .unwrap();
    let ImageData::Raw {
        alpha, gray: false, ..
    } = &store.get(see_through).data
    else {
        panic!()
    };
    assert!(alpha.is_some());
    assert!(matches!(
        store.get(opaque).data,
        ImageData::Raw { alpha: None, .. }
    ));
    assert!(matches!(
        store.get(gray).data,
        ImageData::Raw { gray: true, .. }
    ));
    assert_eq!(store.size(see_through), (1, 2));
    assert!(store.add(b"nope").is_err());
}

#[test]
fn jpeg_passes_through_with_exif_orientation() {
    // SOI, APP1 Exif (big endian, one entry: orientation 6), SOF0 40x30 RGB, EOI.
    let mut jpg = vec![0xFF, 0xD8];
    let tiff = [
        b"MM\0\x2A\0\0\0\x08".as_slice(),
        &[0, 1, 0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0],
        &[0; 4],
    ]
    .concat();
    let app1 = [b"Exif\0\0".as_slice(), &tiff].concat();
    jpg.extend([0xFF, 0xE1]);
    jpg.extend(((app1.len() + 2) as u16).to_be_bytes());
    jpg.extend(app1);
    jpg.extend([
        0xFF, 0xC0, 0, 11, 8, 0, 30, 0, 40, 3, 0, 0, 0, 0, 0xFF, 0xD9,
    ]);
    let mut store = ImageStore::new();
    let id = store.add(&jpg).unwrap();
    assert_eq!(store.get(id).orientation, 6);
    assert_eq!(store.size(id), (30, 40));
    assert!(matches!(
        store.get(id).data,
        ImageData::Jpeg { components: 3, .. }
    ));
}

#[test]
fn placement_turns_the_image_into_its_box() {
    let map =
        |m: [f32; 6], u: f32, v: f32| (m[0] * u + m[2] * v + m[4], m[1] * u + m[3] * v + m[5]);
    // The image's top left pixel (u 0, v 1) lands where each orientation puts it.
    let corners = [
        (1, (0., 0.)),
        (2, (10., 0.)),
        (3, (10., 20.)),
        (4, (0., 20.)),
        (5, (0., 0.)),
        (6, (10., 0.)),
        (7, (10., 20.)),
        (8, (0., 20.)),
    ];
    for (o, want) in corners {
        assert_eq!(
            map(placement(o, 0., 0., 10., 20.), 0., 1.),
            want,
            "orientation {o}"
        );
    }
}
