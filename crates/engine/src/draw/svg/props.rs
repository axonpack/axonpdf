//! An SVG node's props after inheritance: what it draws with.

use serde_json::{Map, Value};

use crate::display::{Color, FillRule, LineCap, LineJoin};
use crate::style::units::parse_float;
use crate::tree::Node;

/// Presentation props children take from their parent unless they set their own.
const INHERITED: [&str; 27] = [
    "x",
    "y",
    "clipPath",
    "clipRule",
    "opacity",
    "fill",
    "fillOpacity",
    "fillRule",
    "stroke",
    "strokeLinecap",
    "strokeLinejoin",
    "strokeOpacity",
    "strokeWidth",
    "textAnchor",
    "dominantBaseline",
    "color",
    "fontFamily",
    "fontSize",
    "fontStyle",
    "fontWeight",
    "fontFeatureSettings",
    "letterSpacing",
    "textDecoration",
    "lineHeight",
    "textAlign",
    "visibility",
    "wordSpacing",
];

const VERTICAL: [&str; 6] = ["y", "y1", "y2", "height", "cy", "ry"];
const HORIZONTAL: [&str; 6] = ["x", "x1", "x2", "width", "cx", "rx"];

#[derive(Clone, Debug, Default)]
pub struct Props {
    pub map: Map<String, Value>,
    /// The size percentages of x and y lengths refer to.
    pub container: (f64, f64),
}

impl Props {
    /// `node`'s own props over what it inherits. Text styles written in `style` count as
    /// props the node did not set.
    pub fn of(node: &Node, inherited: &Map<String, Value>, container: (f64, f64)) -> Props {
        let mut map = inherited.clone();
        let s = &node.style;
        let from_style: [(&str, Option<Value>); 5] = [
            ("fontSize", s.font_size.map(Value::from)),
            ("fontFamily", s.font_family.clone().map(Value::from)),
            ("fontWeight", s.font_weight.map(Value::from)),
            ("fontStyle", s.font_style.clone().map(Value::from)),
            ("opacity", s.opacity.map(Value::from)),
        ];
        for (k, v) in from_style {
            if let Some(v) = v {
                map.insert(k.into(), v);
            }
        }
        for (k, v) in &node.props {
            // `none` reads as unset.
            let v = if v.as_str() == Some("none") { Value::Null } else { v.clone() };
            map.insert(k.clone(), v);
        }
        Props { map, container }
    }

    /// What this node's children inherit. A `Text` keeps its `x` to itself.
    pub fn inherited(&self, is_text: bool) -> Map<String, Value> {
        INHERITED
            .iter()
            .filter(|k| !(is_text && **k == "x"))
            .filter_map(|k| self.map.get(*k).map(|v| (k.to_string(), v.clone())))
            .collect()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key).filter(|v| !v.is_null())
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    /// A length: a number, a number in a string, or a percentage of the container.
    pub fn num(&self, key: &str) -> Option<f64> {
        let v = self.get(key)?;
        if let Some(p) = v.as_str().and_then(|s| s.trim().strip_suffix('%')) {
            let p = p.trim().parse::<f64>().ok()? / 100.0;
            if VERTICAL.contains(&key) {
                return Some(p * self.container.1);
            }
            if HORIZONTAL.contains(&key) {
                return Some(p * self.container.0);
            }
            return Some(p);
        }
        parse_float(v)
    }

    pub fn n(&self, key: &str) -> f64 {
        self.num(key).unwrap_or(0.0)
    }

    /// Whether a prop is set to something drawable (not missing, `none`, empty or zero).
    pub fn is(&self, key: &str) -> bool {
        self.get(key).is_some_and(crate::tree::truthy)
    }

    pub fn color(&self, key: &str) -> Option<Color> {
        self.str(key).map(crate::style::color::parse)
    }

    /// `url(#id)` references.
    pub fn url(&self, key: &str) -> Option<&str> {
        let s = self.str(key)?.trim();
        let inner = s.strip_prefix("url(")?.strip_suffix(')')?;
        Some(inner.trim_matches(|c| c == '\'' || c == '"').trim_start_matches('#'))
    }

    /// Fill and stroke alpha: `opacity` wins over `fillOpacity` and `strokeOpacity`.
    pub fn alphas(&self) -> (f32, f32) {
        let opacity = self.num("opacity");
        // A zero fill opacity reads as unset; a zero stroke opacity does not.
        let fill = opacity.or(self.num("fillOpacity").filter(|v| *v != 0.0)).unwrap_or(1.0);
        let stroke = opacity.or(self.num("strokeOpacity")).unwrap_or(1.0);
        (fill as f32, stroke as f32)
    }

    pub fn fill_rule(&self) -> FillRule {
        match self.str("fillRule") {
            Some("evenodd") => FillRule::EvenOdd,
            _ => FillRule::NonZero,
        }
    }

    pub fn cap(&self) -> LineCap {
        match self.str("strokeLinecap") {
            Some("round") => LineCap::Round,
            Some("square") => LineCap::Square,
            _ => LineCap::Butt,
        }
    }

    pub fn join(&self) -> LineJoin {
        match self.str("strokeLinejoin") {
            Some("round") => LineJoin::Round,
            Some("bevel") => LineJoin::Bevel,
            _ => LineJoin::Miter,
        }
    }

    pub fn dash(&self) -> Vec<f32> {
        match self.get("strokeDasharray") {
            Some(Value::String(s)) => s
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|p| !p.is_empty())
                .map(|p| p.parse::<f32>().unwrap_or(0.0))
                .collect(),
            Some(Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0) as f32],
            Some(Value::Array(a)) => a.iter().filter_map(Value::as_f64).map(|v| v as f32).collect(),
            _ => Vec::new(),
        }
    }
}
