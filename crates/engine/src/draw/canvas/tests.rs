//! Paints the calls a typical `paint` function records into `target/tmp/canvas.pdf`.

use serde_json::json;

use super::draw;
use crate::display::{Op, PageOut};
use crate::fonts::FontStore;
use crate::images::ImageStore;
use crate::writer::{DocInfo, Writer};

#[test]
fn paints_recorded_calls() {
    let calls = json!([
        ["save"], ["rect", 10, 10, 80, 60], ["fillColor", "#00afb9"], ["fill"],
        ["roundedRect", 100, 10, 80, 60, 12], ["lineWidth", 3], ["strokeColor", "#f07167"], ["stroke"],
        ["circle", 230, 40, 30], ["fillOpacity", 0.5], ["fillColor", "#0081a7"], ["fill"], ["fillOpacity", 1],
        ["save"], ["moveTo", 280, 70], ["bezierCurveTo", 300, 0, 340, 0, 360, 70], ["quadraticCurveTo", 380, 30, 400, 70],
        ["lineWidth", 2], ["dash", 5, {"space": 3}], ["stroke"], ["restore"],
        ["polygon", [420, 70], [450, 10], [480, 70]], ["fillAndStroke", "#fed9b7", "#333"],
        ["linearGradient", 0, 500, 0, 530, 0], ["stop", 0, 0, "red"], ["stop", 0, 1, "blue", 0.5],
        ["rect", 500, 10, 30, 60], ["fill", {"gradient": 0}],
        ["restore"], ["fontSize", 10], ["fillColor", "#333"], ["text", "paint got 535 x 90", 10, 76],
        ["rotate", 30, {"origin": [500, 80]}], ["nonsense", 1]
    ]);
    let fonts = FontStore::new();
    let mut ops = Vec::new();
    draw(&mut ops, calls.as_array().unwrap(), 30.0, 30.0, &fonts);
    let count = |f: fn(&Op) -> bool| ops.iter().filter(|o| f(o)).count();
    assert_eq!(count(|o| matches!(o, Op::Path { .. })), 6);
    assert_eq!(count(|o| matches!(o, Op::Text(_))), 1);
    assert_eq!(
        count(|o| matches!(o, Op::Save)),
        count(|o| matches!(o, Op::Restore))
    );

    let mut pdf = Vec::new();
    let mut out = |c: &[u8]| pdf.extend_from_slice(c);
    let mut w = Writer::new(DocInfo::default());
    w.begin(&mut out);
    w.write_page(
        &PageOut {
            width: 595.0,
            height: 150.0,
            ops,
        },
        &fonts,
        &mut ImageStore::new(),
        &mut out,
    );
    w.finish(&fonts, &mut out);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("canvas.pdf"), &pdf).unwrap();
}
