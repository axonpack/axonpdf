mod common;

use axonpdf::display::{Deferred, DeferredKey, Op, PageOut, TextAlign};
use axonpdf::writer::Writer;

/// A 30-page document whose footers say "Page n of <total>", the total drawn after the last
/// page: in Helvetica on even pages, in an embedded subset font on odd ones.
#[test]
fn page_total_is_filled_in_at_the_end() {
    let mut a = common::assets();
    let helvetica = a.fonts.resolve("Helvetica", 400, false).unwrap();
    let mut pdf = Vec::new();
    let mut out = |c: &[u8]| pdf.extend_from_slice(c);
    let mut w = Writer::new(common::info());
    w.begin(&mut out);
    for n in 1..=30 {
        let label = format!("Page {n} of ");
        let width = a.fonts.measure(helvetica, &label, 10.0, 0.0);
        let font = if n % 2 == 0 { helvetica } else { a.lato };
        let ops = vec![
            common::text(
                &a.fonts,
                helvetica,
                &label,
                10.0,
                40.0,
                800.0,
                common::rgb(0.2, 0.2, 0.2),
            ),
            Op::Deferred(Deferred {
                key: DeferredKey::TotalPages,
                font,
                size: 10.0,
                color: common::rgb(0.2, 0.2, 0.2),
                x: 40.0 + width,
                baseline: 800.0,
                width: 30.0,
                align: TextAlign::Left,
            }),
        ];
        w.write_page(
            &PageOut {
                width: 595.0,
                height: 842.0,
                ops,
            },
            &a.fonts,
            &mut a.images,
            &mut out,
        );
    }
    w.finish(&a.fonts, &mut out);
    let count = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();
    assert_eq!(count(b"/Subtype /Form"), 2, "one shared form per font");

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("deferred.pdf");
    std::fs::write(&path, &pdf).unwrap();
    let p = path.to_str().unwrap();
    if let Some(r) = common::tool("qpdf", &["--check", p]) {
        let s = String::from_utf8_lossy(&r.stdout);
        assert!(r.status.success() && !s.contains("WARNING"), "{s}");
    }
    let Some(r) = common::tool("pdftotext", &["-layout", p, "-"]) else {
        return;
    };
    let text = String::from_utf8_lossy(&r.stdout);
    let pages: Vec<&str> = text
        .split('\u{c}')
        .filter(|s| !s.trim().is_empty())
        .collect();
    assert_eq!(pages.len(), 30);
    for (i, page) in pages.iter().enumerate() {
        assert!(
            page.contains(&format!("Page {} of 30", i + 1)),
            "page {}: {page:?}",
            i + 1
        );
    }
}
