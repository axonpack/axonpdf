mod common;

use axonpdf::display::*;
use axonpdf::writer::Writer;

fn field(name: &str, y: f32, parent: Option<u32>, kind: FieldKind) -> Op {
    Op::Field(Field {
        rect: Rect {
            x: 40.0,
            y,
            w: 200.0,
            h: 20.0,
        },
        name: name.into(),
        parent,
        kind,
        required: true,
        no_export: false,
        read_only: false,
        value: Some("Ada".into()),
        default_value: None,
    })
}

/// A 14pt-high field, so its looks are not shared with the 20pt ones.
fn small(mut op: Op) -> Op {
    if let Op::Field(f) = &mut op {
        f.rect.h = 14.0;
    }
    op
}

#[test]
fn form_fields_make_a_valid_acroform() {
    let mut a = common::assets();
    let text = FieldKind::Text {
        align: Align::Center,
        multiline: false,
        password: false,
        no_spell: true,
        format: Some(FieldFormat::Number {
            decimals: 2,
            sep_comma: true,
            neg_style: "MinusBlack".into(),
            currency: "$".into(),
            currency_prepend: true,
        }),
        font_size: Some(12.0),
        max_len: Some(20),
    };
    let check = |checked, x_mark| FieldKind::Checkbox {
        checked,
        on_state: "Yes".into(),
        off_state: "Off".into(),
        x_mark,
        background: Some(common::rgb(1.0, 1.0, 0.8)),
        border: None,
    };
    let choice = |combo| FieldKind::Choice {
        combo,
        options: vec!["One".into(), "Two".into()],
        sort: false,
        edit: combo,
        multi_select: !combo,
        no_spell: false,
    };
    let page = PageOut {
        width: 300.0,
        height: 300.0,
        ops: vec![
            Op::FieldSet {
                id: 1,
                name: "person".into(),
                parent: None,
            },
            field("name", 20.0, Some(1), text),
            field("agree", 50.0, Some(1), check(true, false)),
            small(field("cross", 80.0, None, check(false, true))),
            field("pick", 110.0, None, choice(true)),
            field("list", 140.0, None, choice(false)),
        ],
    };
    let mut pdf = Vec::new();
    let mut out = |c: &[u8]| pdf.extend_from_slice(c);
    let mut w = Writer::new(common::info());
    w.begin(&mut out);
    w.write_page(&page, &a.fonts, &mut a.images, &mut out);
    w.finish(&a.fonts, &mut out);
    // Looks: a check for "agree", a cross for "cross", and unchecked blank for both sizes.
    let count = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();
    assert_eq!(
        (count(b"(4) Tj"), count(b"(8) Tj"), count(b"( ) Tj")),
        (1, 1, 2)
    );
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("forms.pdf");
    std::fs::write(&path, &pdf).unwrap();
    if let Some(r) = common::tool("qpdf", &["--check", path.to_str().unwrap()]) {
        let s = String::from_utf8_lossy(&r.stdout);
        assert!(r.status.success() && !s.contains("WARNING"), "{s}");
    }
    if let Some(r) = common::tool(
        "qpdf",
        &["--json", "--json-key=acroform", path.to_str().unwrap()],
    ) {
        let json = String::from_utf8_lossy(&r.stdout);
        for name in ["person.name", "person.agree", "cross", "pick", "list"] {
            assert!(
                json.contains(&format!("\"fullname\": \"{name}\"")),
                "{name} missing:\n{json}"
            );
        }
    }
}
