mod common;

use axonpdf::writer::{Conformance, Writer};

#[test]
fn pdfa_adds_xmp_and_an_output_intent() {
    assert_eq!(
        Conformance::parse("PDF/A-2b"),
        Some(Conformance {
            part: 2,
            level: 'B'
        })
    );
    assert_eq!(
        Conformance::parse("PDF/A-1"),
        Some(Conformance {
            part: 1,
            level: 'B'
        })
    );
    assert_eq!(Conformance::parse("PDF/X-1"), None);
    let mut a = common::assets();
    let mut info = common::info();
    info.conformance = Conformance::parse("PDF/A-2b");
    let mut pdf = Vec::new();
    let mut out = |c: &[u8]| pdf.extend_from_slice(c);
    let mut w = Writer::new(info);
    w.begin(&mut out);
    w.write_page(
        &common::sample_page(&a, 1),
        &a.fonts,
        &mut a.images,
        &mut out,
    );
    w.finish(&a.fonts, &mut out);
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.starts_with("%PDF-1.7"));
    for needle in [
        "<pdfaid:part>2</pdfaid:part>",
        "<pdfaid:conformance>B</pdfaid:conformance>",
        "/OutputIntents",
        "/GTS_PDFA1",
        "/Metadata",
        "<dc:title>",
    ] {
        assert!(text.contains(needle), "{needle}");
    }
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("pdfa.pdf");
    std::fs::write(&path, &pdf).unwrap();
    if let Some(r) = common::tool("qpdf", &["--check", path.to_str().unwrap()]) {
        let s = String::from_utf8_lossy(&r.stdout);
        assert!(r.status.success() && !s.contains("WARNING"), "{s}");
    }
}
