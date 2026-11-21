//! `Note` and the form components (`FieldSet`, `TextInput`, `Checkbox`, `Select`, `List`) into
//! ops. Fields inside field sets belong to the outermost one; selects and lists never belong to
//! a set; nested sets are listed on their own.

use std::cell::{Cell, RefCell};

use serde_json::Value;

use crate::display::{Align, Field, FieldFormat, FieldKind, Op, Rect};
use crate::tree::{Kind, Node};

/// The field sets open around the node being drawn.
#[derive(Default)]
pub struct Forms {
    open: RefCell<Vec<u32>>,
    next: Cell<u32>,
}

/// Draws a note or a field whose border box is (`x`, `y`, `w`, `h`), or opens a field set.
pub fn enter(ops: &mut Vec<Op>, n: &Node, (x, y, w, h): (f64, f64, f64, f64), forms: &Forms) {
    let kind = match &n.kind {
        Kind::Note => {
            let text = n
                .children
                .first()
                .map(|c| c.value.clone())
                .unwrap_or_default();
            ops.push(Op::Note {
                x: x as f32,
                y: y as f32,
                text,
                color: n.style.background_color,
            });
            return;
        }
        Kind::Other(k) => k.as_str(),
        _ => return,
    };
    let str_of = |k: &str| {
        n.props
            .get(k)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    let flag = |k: &str| n.is(k);
    let outer = forms.open.borrow().first().copied();
    let kind = match kind {
        "FIELD_SET" => {
            let id = forms.next.get();
            forms.next.set(id + 1);
            forms.open.borrow_mut().push(id);
            ops.push(Op::FieldSet {
                id,
                name: str_of("name").unwrap_or_default(),
                parent: None,
            });
            return;
        }
        "TEXT_INPUT" => FieldKind::Text {
            align: match n.props.get("align").and_then(Value::as_str) {
                Some("center") => Align::Center,
                Some("right") => Align::Right,
                _ => Align::Left,
            },
            multiline: flag("multiline"),
            password: flag("password"),
            no_spell: flag("noSpell"),
            format: n.props.get("format").and_then(format),
            font_size: n.number("fontSize").filter(|s| *s != 0.0).map(|s| s as f32),
            max_len: n.number("maxLength").filter(|m| *m > 0.0).map(|m| m as u32),
        },
        "CHECKBOX" => FieldKind::Checkbox {
            checked: n.props.get("checked") == Some(&Value::Bool(true)),
            on_state: str_of("onState").unwrap_or_else(|| "Yes".into()),
            off_state: str_of("offState").unwrap_or_else(|| "Off".into()),
            x_mark: flag("xMark"),
            background: str_of("backgroundColor").map(|c| crate::style::color::parse(&c)),
            border: str_of("borderColor").map(|c| crate::style::color::parse(&c)),
        },
        "SELECT" | "LIST" => FieldKind::Choice {
            combo: kind == "SELECT",
            options: match n.props.get("select") {
                Some(Value::Array(a)) if !a.is_empty() => a
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect(),
                _ => vec![String::new()],
            },
            sort: flag("sort"),
            edit: flag("edit"),
            multi_select: flag("multiSelect"),
            no_spell: flag("noSpell"),
        },
        _ => return,
    };
    let parent = if matches!(kind, FieldKind::Choice { .. }) {
        None
    } else {
        outer
    };
    ops.push(Op::Field(Field {
        rect: Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        },
        name: str_of("name").unwrap_or_default(),
        parent,
        kind,
        required: flag("required"),
        no_export: flag("noExport"),
        read_only: flag("readOnly"),
        value: str_of("value"),
        default_value: str_of("defaultValue"),
    }));
}

/// Closes a field set after its children.
pub fn leave(n: &Node, forms: &Forms) {
    if matches!(&n.kind, Kind::Other(k) if k == "FIELD_SET") {
        forms.open.borrow_mut().pop();
    }
}

fn format(v: &Value) -> Option<FieldFormat> {
    let num = |k: &str, d: f64| v.get(k).and_then(Value::as_f64).unwrap_or(d);
    let text = |k: &str, d: &str| v.get(k).and_then(Value::as_str).unwrap_or(d).to_string();
    let comma = v.get("sepComma").and_then(Value::as_bool).unwrap_or(false);
    let param = v
        .get("param")
        .map(|p| p.as_str().map_or(p.to_string(), String::from))
        .unwrap_or_default();
    Some(match v.get("type")?.as_str()? {
        "zip" => FieldFormat::Zip,
        "zipPlus4" | "zip4" => FieldFormat::ZipPlus4,
        "phone" => FieldFormat::Phone,
        "ssn" => FieldFormat::Ssn,
        "date" => FieldFormat::Date(param),
        "time" => FieldFormat::Time(param.parse().unwrap_or(0)),
        "number" => FieldFormat::Number {
            decimals: num("nDec", 0.0) as u32,
            sep_comma: comma,
            neg_style: text("negStyle", "MinusBlack"),
            currency: text("currency", ""),
            currency_prepend: v
                .get("currencyPrepend")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        },
        "percent" => FieldFormat::Percent {
            decimals: num("nDec", 0.0) as u32,
            sep_comma: comma,
        },
        _ => return None,
    })
}
