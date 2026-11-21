//! Box and line parity against the reference engine's layout dumps. Local only: set
//! `AXONPDF_PARITY` to a folder holding `<name>.tree.json` (the tree the components send) and
//! `<name>.layout.json` (the reference's layout data), then
//! `cargo test --test parity -- --ignored --nocapture`.

use axonpdf::Engine;
use axonpdf::tree::{Kind, Node};
use serde_json::{Value, json};

fn dump(n: &Node) -> Value {
    let kind = match &n.kind {
        Kind::Other(k) => k.clone(),
        k => format!("{k:?}").to_uppercase(),
    };
    let lines = n
        .text
        .as_ref()
        .and_then(|t| t.borrow().lines.as_ref().map(|l| l.len()))
        .unwrap_or(0);
    let l = &n.layout;
    json!({
        "type": kind,
        "box": [l.left, l.top, l.width, l.height],
        "lines": lines,
        "children": n.children.iter().filter(|c| c.kind != Kind::TextInstance).map(dump).collect::<Vec<_>>(),
    })
}

fn theirs(n: &Value) -> Value {
    let b = &n["box"];
    let num = |k: &str| b[k].as_f64().unwrap_or(0.0);
    let kind = n["type"].as_str().unwrap_or("").replace('_', "");
    json!({
        "type": kind,
        "box": [num("left"), num("top"), num("width"), num("height")],
        "lines": n["lines"].as_array().map_or(0, |l| l.len()),
        "children": n["children"].as_array().map_or(vec![], |c| c.iter()
            .filter(|c| c["type"] != "TEXT_INSTANCE").map(theirs).collect()),
    })
}

fn compare(path: &str, ours: &Value, refr: &Value, out: &mut Vec<String>) {
    let close = |a: &Value, b: &Value| (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 0.01;
    let boxes = ours["box"]
        .as_array()
        .unwrap()
        .iter()
        .zip(refr["box"].as_array().unwrap())
        .all(|(a, b)| close(a, b));
    if ours["type"] != refr["type"] || !boxes || ours["lines"] != refr["lines"] {
        out.push(format!(
            "{path}: ours {} {} lines {}, ref {} {} lines {}",
            ours["type"], ours["box"], ours["lines"], refr["type"], refr["box"], refr["lines"]
        ));
    }
    let (a, b) = (
        ours["children"].as_array().unwrap(),
        refr["children"].as_array().unwrap(),
    );
    if a.len() != b.len() {
        out.push(format!("{path}: {} children, ref {}", a.len(), b.len()));
    }
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        compare(&format!("{path}/{i}"), x, y, out);
    }
}

#[test]
#[ignore]
fn boxes_match_the_reference() {
    let Ok(dir) = std::env::var("AXONPDF_PARITY") else {
        return;
    };
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".tree.json")
        else {
            continue;
        };
        let Ok(layout) = std::fs::read_to_string(format!("{dir}/{name}.layout.json")) else {
            continue;
        };
        let tree = std::fs::read_to_string(&path).unwrap();
        let refr: Value = serde_json::from_str(&layout).unwrap();
        let refr = &refr["_INTERNAL__LAYOUT__DATA_"]["children"];
        let mut engine = Engine::new();
        engine.set_render_callback(Box::new(|_, a| {
            json!({"children": [{"type": "TEXT_INSTANCE", "value": format!("Page {}", a.page_number)}], "readsTotal": false}).to_string()
        }));
        let mut pages = Vec::new();
        if let Err(e) = engine.layout(&tree, &mut |p, _, _| pages.push(dump(p))) {
            println!("{name}: {e}");
            continue;
        }
        let refs: Vec<Value> = refr.as_array().unwrap().iter().map(theirs).collect();
        let mut diffs = Vec::new();
        if pages.len() != refs.len() {
            diffs.push(format!("{} pages, ref {}", pages.len(), refs.len()));
        }
        for (i, (a, b)) in pages.iter().zip(&refs).enumerate() {
            compare(&format!("page {}", i + 1), a, b, &mut diffs);
        }
        println!("== {name}: {} differences", diffs.len());
        for d in diffs.iter().take(25) {
            println!("  {d}");
        }
    }
}
