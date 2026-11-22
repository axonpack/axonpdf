use super::*;
use crate::fonts::FontStore;

fn attrs(size: f64, align: &str) -> Rc<Attrs> {
    Rc::new(Attrs {
        size,
        color: Color::BLACK,
        opacity: None,
        letter_spacing: 0.0,
        line_height: None,
        underline: false,
        strike: false,
        decoration_color: Color::BLACK,
        decoration_style: "solid".into(),
        link: None,
        align: align.into(),
        background: None,
        indent: 0.0,
        vertical_align: None,
        attachment: None,
        deferred: false,
        rtl: false,
        features: Vec::new(),
    })
}

fn lines(text: &str, align: &str, width: f64) -> Vec<Line> {
    let fonts = FontStore::new();
    let helvetica = fonts.resolve("Helvetica", 400, false).unwrap();
    let frags = [Fragment {
        text: text.into(),
        attrs: attrs(12.0, align),
        fonts: vec![helvetica],
    }];
    layout(
        &prepare(&fonts, &frags, &mut hyphen::builtin),
        width,
        None,
        &Options::default(),
    )
}

fn text(l: &Line) -> String {
    l.runs.iter().map(|r| r.text.as_str()).collect()
}

const QUICK: &str =
    "The quick brown fox jumps over the lazy dog. Typesetting hyphenation extraordinarily well.";

// Expected values are what the reference engine lays out for the same text.
#[test]
fn breaks_and_hyphenates_like_the_reference() {
    let ls = lines(QUICK, "left", 200.0);
    let got: Vec<_> = ls
        .iter()
        .map(|l| (text(l), l.x_advance, l.width, l.y, l.height))
        .collect();
    let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
    assert_eq!(got.len(), 3);
    assert_eq!(got[0].0, "The quick brown fox jumps over the ");
    assert!(close(got[0].1, 191.124) && close(got[0].2, 203.336));
    assert_eq!(got[1].0, "lazy dog. Typesetting hyphenation ex-");
    assert!(close(got[1].1, 199.428) && close(got[1].2, 200.0));
    assert_eq!(got[2].0, "traordinarily well.");
    assert!(close(got[2].1, 89.952));
    assert!(close(got[2].3, 26.4) && close(got[2].4, 13.2));
    assert!(close(ls[0].ascent, 10.8));
}

#[test]
fn justifies_all_but_the_last_line() {
    let ls = lines(QUICK, "justify", 200.0);
    let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
    assert!(close(ls[0].x_advance, 203.336));
    assert!(close(ls[1].x_advance, 200.0));
    assert!(close(ls[2].x_advance, 89.952));
}

#[test]
fn runs_split_where_the_script_changes() {
    let fonts = FontStore::new();
    let helvetica = fonts.resolve("Helvetica", 400, false).unwrap();
    let frags = [Fragment {
        text: "(abc Ωμέγα, def".into(),
        attrs: attrs(12.0, "left"),
        fonts: vec![helvetica],
    }];
    let p = &prepare(&fonts, &frags, &mut hyphen::builtin)[0];
    let runs: Vec<String> = p
        .runs
        .iter()
        .map(|r| p.chars[r.start..r.end].iter().collect())
        .collect();
    // The leading bracket has no script of its own; spaces and the comma stay with the script
    // before them.
    assert_eq!(runs, ["(", "abc ", "Ωμέγα, ", "def"]);
}

#[test]
fn font_feature_settings_change_the_glyphs() {
    let mut fonts = FontStore::new();
    let lato = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/assets/Lato-Regular.ttf"));
    let lato = fonts.add_font("Lato", 400, false, lato.unwrap()).unwrap();
    let ids = |text: &str, features: Vec<([u8; 4], bool)>| -> Vec<u16> {
        let a = Rc::new(Attrs { features, ..(*attrs(12.0, "left")).clone() });
        let frags = [Fragment { text: text.into(), attrs: a, fonts: vec![lato] }];
        let p = &prepare(&fonts, &frags, &mut |w: &str| vec![w.to_string()])[0];
        p.runs.iter().flat_map(|r| r.glyphs.iter().map(|g| g.id)).collect()
    };
    // `fi` is one ligature glyph by default; `liga` off keeps two.
    assert_eq!(ids("fi", vec![]).len(), 1);
    assert_eq!(ids("fi", vec![(*b"liga", false)]).len(), 2);
    // Old-style figures swap every digit for another glyph.
    let lining = ids("2026", vec![]);
    let old_style = ids("2026", vec![(*b"onum", true)]);
    assert!(lining.iter().zip(&old_style).all(|(a, b)| a != b));
}
