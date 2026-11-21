mod common;

use std::path::PathBuf;

fn save(name: &str, bytes: &[u8]) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn every_op_makes_a_valid_file() {
    let pdf = common::render(3);
    let path = save("every-op.pdf", &pdf);
    let p = path.to_str().unwrap();
    if let Some(check) = common::tool("qpdf", &["--check", p]) {
        let report = String::from_utf8_lossy(&check.stdout);
        assert!(
            check.status.success() && !report.contains("WARNING"),
            "qpdf:\n{report}{}",
            String::from_utf8_lossy(&check.stderr)
        );
    }
    let png = path.with_extension("");
    if let Some(r) = common::tool(
        "pdftoppm",
        &[
            "-png",
            "-r",
            "72",
            "-f",
            "1",
            "-l",
            "1",
            p,
            png.to_str().unwrap(),
        ],
    ) {
        assert!(
            r.status.success() && r.stderr.is_empty(),
            "pdftoppm: {}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}

#[test]
fn images_and_fonts_are_written_once() {
    let pdf = common::render(5);
    let count = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();
    assert_eq!(count(b"/Subtype /Image"), 3, "badge, its alpha mask, photo");
    assert_eq!(count(b"/Type /Page\n"), 5);
    assert_eq!(count(b"/FontFile2"), 1);
}

#[test]
fn bookmarks_point_at_their_left_and_top() {
    let pdf = common::render(1);
    // The sample's first bookmark sits at left 0, top 0 of an 841.89pt page.
    assert!(String::from_utf8_lossy(&pdf).contains("/XYZ 0 841.89 0]"));
}
