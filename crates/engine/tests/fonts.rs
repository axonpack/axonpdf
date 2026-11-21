use axonpdf::display::FontId;
use axonpdf::fonts::{FontError, FontStore};

#[test]
fn helvetica_widths_and_kerning() {
    let store = FontStore::new();
    let helvetica = store.resolve("Helvetica", 400, false).unwrap();
    let glyphs = store.shape(helvetica, "AV", 10.0, 0.0, &[]);
    assert_eq!(glyphs[0].id, 65);
    assert_eq!(glyphs[0].x_advance, (667.0 - 70.0) / 100.0);
    assert_eq!(glyphs[1].x_advance, 6.67);
    assert!((store.measure(helvetica, "AV", 10.0, 1.0) - (13.34 - 0.7 + 2.0)).abs() < 1e-5);
    // Typographic quotes and the euro sign come from WinAnsi's 0x80 block.
    assert_eq!(
        store.shape(helvetica, "\u{201C}€", 10.0, 0.0, &[])[0].id,
        147
    );
    assert_eq!(store.metrics(helvetica).ascent, 900.0);
    assert_eq!(
        store
            .metrics(store.resolve("Times-Roman", 400, false).unwrap())
            .descent,
        -220.0
    );
}

#[test]
fn families_resolve_like_the_built_in_store() {
    let store = FontStore::new();
    assert_eq!(store.resolve("Helvetica", 700, true), Ok(FontId(3)));
    assert_eq!(store.resolve("Courier", 900, false), Ok(FontId(9)));
    assert_eq!(store.resolve("Times-Bold", 400, false), Ok(FontId(5)));
    assert_eq!(
        store.resolve("Times-Bold", 400, true),
        Err(FontError::NoMatch {
            family: "Times-Bold".into(),
            weight: 400,
            italic: true
        })
    );
    assert_eq!(
        store.resolve("Lato", 400, false),
        Err(FontError::NotRegistered("Lato".into()))
    );
}

#[test]
fn registered_font_shapes_with_kerning_and_ligatures() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/assets/Lato-Regular.ttf"
    );
    let mut store = FontStore::new();
    let lato = store
        .add_font("Lato", 400, false, std::fs::read(path).unwrap())
        .unwrap();
    assert_eq!(store.resolve("Lato", 700, false), Ok(lato));
    assert!(store.has_char(lato, 'é'));
    let plain = store.shape(lato, "AV", 10.0, 0.0, &[]);
    let natural = store.advance(lato, plain[0].id) * 10.0 / store.metrics(lato).units_per_em;
    assert!(plain[0].x_advance < natural, "kerned");
    let fi = store.shape(lato, "fi", 10.0, 0.0, &[]);
    let no_liga = store.shape(lato, "fi", 10.0, 0.0, &[(*b"liga", false)]);
    assert_eq!((fi.len(), no_liga.len()), (1, 2));
}
