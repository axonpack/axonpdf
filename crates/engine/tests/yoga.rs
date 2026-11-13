//! Every case in `fixtures/yoga.json` must lay out exactly as yoga-layout 3.2.1 laid it out
//! (`parity/yoga.mjs` writes those answers), and every Yoga style binding must be exercised by at
//! least one case.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

use axonpdf::layout::yoga::*;
use serde_json::Value;

const FIXTURES: &str = include_str!("fixtures/yoga.json");
const BINDINGS: &str = include_str!("../src/layout/yoga.rs");

const FLEX_DIRECTIONS: [(&str, FlexDirection); 4] = [
    ("column", FlexDirection::Column),
    ("column-reverse", FlexDirection::ColumnReverse),
    ("row", FlexDirection::Row),
    ("row-reverse", FlexDirection::RowReverse),
];
const JUSTIFIES: [(&str, Justify); 6] = [
    ("flex-start", Justify::FlexStart),
    ("center", Justify::Center),
    ("flex-end", Justify::FlexEnd),
    ("space-between", Justify::SpaceBetween),
    ("space-around", Justify::SpaceAround),
    ("space-evenly", Justify::SpaceEvenly),
];
const ALIGNS: [(&str, Align); 9] = [
    ("auto", Align::Auto),
    ("flex-start", Align::FlexStart),
    ("center", Align::Center),
    ("flex-end", Align::FlexEnd),
    ("stretch", Align::Stretch),
    ("baseline", Align::Baseline),
    ("space-between", Align::SpaceBetween),
    ("space-around", Align::SpaceAround),
    ("space-evenly", Align::SpaceEvenly),
];
const POSITION_TYPES: [(&str, PositionType); 3] = [
    ("static", PositionType::Static),
    ("relative", PositionType::Relative),
    ("absolute", PositionType::Absolute),
];
const WRAPS: [(&str, Wrap); 3] = [
    ("no-wrap", Wrap::NoWrap),
    ("wrap", Wrap::Wrap),
    ("wrap-reverse", Wrap::WrapReverse),
];
const OVERFLOWS: [(&str, Overflow); 3] = [
    ("visible", Overflow::Visible),
    ("hidden", Overflow::Hidden),
    ("scroll", Overflow::Scroll),
];
const DISPLAYS: [(&str, Display); 3] = [
    ("flex", Display::Flex),
    ("none", Display::None),
    ("contents", Display::Contents),
];
const EDGES: [(&str, Edge); 9] = [
    ("left", Edge::Left),
    ("top", Edge::Top),
    ("right", Edge::Right),
    ("bottom", Edge::Bottom),
    ("start", Edge::Start),
    ("end", Edge::End),
    ("horizontal", Edge::Horizontal),
    ("vertical", Edge::Vertical),
    ("all", Edge::All),
];
const GUTTERS: [(&str, Gutter); 3] = [
    ("column", Gutter::Column),
    ("row", Gutter::Row),
    ("all", Gutter::All),
];

/// What the cases touched: Yoga functions by name, and enum values as `kind:name`.
#[derive(Default)]
struct Seen(BTreeSet<String>);

impl Seen {
    fn binding(&mut self, key: &str, suffix: &str) {
        let camel: String = key
            .split('_')
            .map(|w| w[..1].to_uppercase() + &w[1..])
            .collect();
        self.0.insert(format!("YGNodeStyleSet{camel}{suffix}"));
    }
    fn value(&mut self, kind: &str, name: &str) {
        self.0.insert(format!("{kind}:{name}"));
    }
}

fn pick<T: Copy>(table: &[(&str, T)], name: &str, kind: &str, seen: &mut Seen) -> T {
    seen.value(kind, name);
    table
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("unknown {kind} {name}"))
        .1
}

fn dim(v: &Value) -> (Dim, &'static str) {
    match v {
        Value::String(s) if s == "auto" => (Dim::Auto, "Auto"),
        Value::String(s) => (
            Dim::Pct(s.trim_end_matches('%').parse().unwrap()),
            "Percent",
        ),
        n => (Dim::Pt(n.as_f64().unwrap() as f32), ""),
    }
}

fn len(v: &Value) -> (Len, &'static str) {
    match dim(v) {
        (Dim::Pt(x), s) => (Len::Pt(x), s),
        (Dim::Pct(x), s) => (Len::Pct(x), s),
        (Dim::Auto, _) => panic!("auto is not allowed here"),
    }
}

fn apply(t: &mut Tree, id: NodeId, key: &str, v: &Value, seen: &mut Seen) {
    let s = v.as_str().unwrap_or("");
    let num = || v.as_f64().unwrap() as f32;
    match key {
        "flex_direction" => t.flex_direction(id, pick(&FLEX_DIRECTIONS, s, key, seen)),
        "justify_content" => t.justify_content(id, pick(&JUSTIFIES, s, key, seen)),
        "align_items" => t.align_items(id, pick(&ALIGNS, s, "align", seen)),
        "align_self" => t.align_self(id, pick(&ALIGNS, s, "align", seen)),
        "align_content" => t.align_content(id, pick(&ALIGNS, s, "align", seen)),
        "position_type" => t.position_type(id, pick(&POSITION_TYPES, s, key, seen)),
        "flex_wrap" => t.flex_wrap(id, pick(&WRAPS, s, key, seen)),
        "overflow" => t.overflow(id, pick(&OVERFLOWS, s, key, seen)),
        "display" => t.display(id, pick(&DISPLAYS, s, key, seen)),
        "flex_grow" => t.flex_grow(id, num()),
        "flex_shrink" => t.flex_shrink(id, num()),
        "aspect_ratio" => t.aspect_ratio(id, num()),
        "flex_basis" | "width" | "height" => {
            let (d, suffix) = dim(v);
            seen.binding(key, suffix);
            match key {
                "flex_basis" => t.flex_basis(id, d),
                "width" => t.width(id, d),
                _ => t.height(id, d),
            }
            return;
        }
        "min_width" | "min_height" | "max_width" | "max_height" => {
            let (l, suffix) = len(v);
            seen.binding(key, suffix);
            match key {
                "min_width" => t.min_width(id, l),
                "min_height" => t.min_height(id, l),
                "max_width" => t.max_width(id, l),
                _ => t.max_height(id, l),
            }
            return;
        }
        "margin" | "padding" | "border" | "position" => {
            for (edge, value) in v.as_object().unwrap() {
                let e = pick(&EDGES, edge, "edge", seen);
                match key {
                    "margin" => {
                        let (d, suffix) = dim(value);
                        seen.binding(key, suffix);
                        t.margin(id, e, d);
                    }
                    "position" => {
                        let (d, suffix) = dim(value);
                        seen.binding(key, suffix);
                        t.position(id, e, d);
                    }
                    "padding" => {
                        let (l, suffix) = len(value);
                        seen.binding(key, suffix);
                        t.padding(id, e, l);
                    }
                    _ => {
                        seen.binding(key, "");
                        t.border(id, e, value.as_f64().unwrap() as f32);
                    }
                }
            }
            return;
        }
        "gap" => {
            for (gutter, value) in v.as_object().unwrap() {
                let g = pick(&GUTTERS, gutter, "gutter", seen);
                let (l, suffix) = len(value);
                seen.binding(key, suffix);
                t.gap(id, g, l);
            }
            return;
        }
        other => panic!("fixture uses unknown style key {other}"),
    }
    seen.binding(key, "");
}

/// A measured leaf returns its own size, clamped by what Yoga offers (same rule as parity/yoga.mjs).
fn fit(offer: f32, mode: MeasureMode, own: f32) -> f32 {
    match mode {
        MeasureMode::Exactly => offer,
        MeasureMode::AtMost => offer.min(own),
        MeasureMode::Undefined => own,
    }
}

fn build(
    t: &mut Tree,
    spec: &Value,
    order: &mut Vec<NodeId>,
    calls: &Rc<Cell<u32>>,
    seen: &mut Seen,
) -> NodeId {
    let id = t.node();
    order.push(id);
    if let Some(style) = spec["style"].as_object() {
        for (k, v) in style {
            apply(t, id, k, v, seen);
        }
    }
    if let Some(m) = spec["measure"].as_array() {
        let (mw, mh) = (m[0].as_f64().unwrap() as f32, m[1].as_f64().unwrap() as f32);
        let calls = calls.clone();
        t.set_measure(
            id,
            Box::new(move |w, wm, h, hm| {
                calls.set(calls.get() + 1);
                (fit(w, wm, mw), fit(h, hm, mh))
            }),
        );
    }
    for child in spec["children"].as_array().into_iter().flatten() {
        let c = build(t, child, order, calls, seen);
        t.push_child(id, c);
    }
    id
}

#[test]
fn every_case_matches_yoga_layout_and_every_binding_is_covered() {
    let doc: Value = serde_json::from_str(FIXTURES).unwrap();
    let mut seen = Seen::default();
    let mut failures = Vec::new();

    for case in doc["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let mut t = Tree::new();
        let mut order = Vec::new();
        let calls = Rc::new(Cell::new(0));
        let root = build(&mut t, &case["root"], &mut order, &calls, &mut seen);
        let side = |i: usize| {
            case["available"]
                .get(i)
                .and_then(Value::as_f64)
                .map(|x| x as f32)
        };
        t.calculate(root, side(0), side(1));

        let expected = &case["expected"];
        for (i, &id) in order.iter().enumerate() {
            let r = t.layout(id);
            let got = [r.left, r.top, r.width, r.height].map(f64::from);
            let want: Vec<f64> = expected["boxes"][i]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
                .collect();
            if got[..] != want[..] {
                failures.push(format!(
                    "{name}: node {i} is {got:?}, yoga-layout has {want:?}"
                ));
            }
        }
        if u64::from(calls.get()) != expected["measure_calls"].as_u64().unwrap() {
            failures.push(format!(
                "{name}: {} measure calls, yoga-layout made {}",
                calls.get(),
                expected["measure_calls"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} differences:\n{}",
        failures.len(),
        failures.join("\n")
    );

    let mut missing: Vec<String> = BINDINGS
        .split("fn ")
        .filter_map(|s| s.strip_prefix("YGNodeStyleSet"))
        .map(|s| format!("YGNodeStyleSet{}", &s[..s.find('(').unwrap()]))
        .filter(|b| !seen.0.contains(b))
        .collect();
    let values = [
        ("flex_direction", FLEX_DIRECTIONS.map(|(n, _)| n).to_vec()),
        ("justify_content", JUSTIFIES.map(|(n, _)| n).to_vec()),
        ("align", ALIGNS.map(|(n, _)| n).to_vec()),
        ("position_type", POSITION_TYPES.map(|(n, _)| n).to_vec()),
        ("flex_wrap", WRAPS.map(|(n, _)| n).to_vec()),
        ("overflow", OVERFLOWS.map(|(n, _)| n).to_vec()),
        ("display", DISPLAYS.map(|(n, _)| n).to_vec()),
        ("edge", EDGES.map(|(n, _)| n).to_vec()),
        ("gutter", GUTTERS.map(|(n, _)| n).to_vec()),
    ];
    for (kind, names) in values {
        missing.extend(
            names
                .iter()
                .map(|n| format!("{kind}:{n}"))
                .filter(|k| !seen.0.contains(k)),
        );
    }
    assert!(missing.is_empty(), "no fixture uses: {missing:?}");
}
