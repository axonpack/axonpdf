//! `shapeOutside`: a CSS basic shape, lengths in points and percentages of the float's box kept
//! for layout to resolve.

use super::units::{Container, Dim, dim, unit_str};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Radius {
    ClosestSide,
    FarthestSide,
    Len(Dim),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShapeOutside {
    Circle {
        cx: Dim,
        cy: Dim,
        r: Radius,
    },
    Ellipse {
        cx: Dim,
        cy: Dim,
        rx: Radius,
        ry: Radius,
    },
    Polygon(Vec<(Dim, Dim)>),
    /// Top, right, bottom and left offsets.
    Inset([Dim; 4]),
}

/// `circle()`, `ellipse()`, `polygon()` or `inset()`; anything else (`url()`, a box keyword,
/// a malformed shape) is `None`, which leaves the float's plain box.
pub fn parse(c: &Container, value: &str) -> Option<ShapeOutside> {
    let (name, args) = value.trim().split_once('(')?;
    let args = args.strip_suffix(')')?;
    let len = |t: &str| dim(&unit_str(c, t)).unwrap_or(Dim::Pt(0.0));
    let radius = |t: Option<&str>| match t {
        None | Some("closest-side") => Radius::ClosestSide,
        Some("farthest-side") => Radius::FarthestSide,
        Some(t) => Radius::Len(len(t)),
    };
    // `<radii> at <position>`: the position defaults to the center, keywords in either order.
    let at = |args: &str| -> (Vec<String>, Dim, Dim) {
        let (radii, pos) = args.split_once(" at ").map_or((args, ""), |(r, p)| (r, p));
        let mut p: Vec<&str> = pos.split_whitespace().collect();
        p.resize(2, "center");
        if matches!(p[0], "top" | "bottom") || matches!(p[1], "left" | "right") {
            p.swap(0, 1);
        }
        let keyword = |t: &str, lo: &str, hi: &str| match t {
            _ if t == lo => Dim::Pct(0.0),
            "center" => Dim::Pct(50.0),
            _ if t == hi => Dim::Pct(100.0),
            _ => len(t),
        };
        let radii = radii.split_whitespace().map(str::to_string).collect();
        (
            radii,
            keyword(p[0], "left", "right"),
            keyword(p[1], "top", "bottom"),
        )
    };
    match name.trim() {
        "circle" => {
            let (r, cx, cy) = at(args);
            let r = radius(r.first().map(String::as_str));
            Some(ShapeOutside::Circle { cx, cy, r })
        }
        "ellipse" => {
            let (r, cx, cy) = at(args);
            let (rx, ry) = (
                radius(r.first().map(String::as_str)),
                radius(r.get(1).map(String::as_str)),
            );
            Some(ShapeOutside::Ellipse { cx, cy, rx, ry })
        }
        "polygon" => {
            let mut parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if matches!(parts.first(), Some(&("nonzero" | "evenodd"))) {
                parts.remove(0);
            }
            let points: Vec<(Dim, Dim)> = parts
                .iter()
                .filter(|p| !p.is_empty())
                .map(|p| {
                    let mut xy = p.split_whitespace();
                    (len(xy.next().unwrap_or("0")), len(xy.next().unwrap_or("0")))
                })
                .collect();
            (points.len() >= 3).then_some(ShapeOutside::Polygon(points))
        }
        "inset" => {
            let t: Vec<Dim> = args
                .split(" round ")
                .next()?
                .split_whitespace()
                .map(len)
                .collect();
            let (top, right) = (*t.first()?, *t.get(1).unwrap_or(&t[0]));
            let bottom = *t.get(2).unwrap_or(&top);
            let left = *t.get(3).unwrap_or(&right);
            Some(ShapeOutside::Inset([top, right, bottom, left]))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_parse_as_written() {
        let c = Container::default();
        assert_eq!(
            parse(&c, "circle(40pt at left center)"),
            Some(ShapeOutside::Circle {
                cx: Dim::Pct(0.0),
                cy: Dim::Pct(50.0),
                r: Radius::Len(Dim::Pt(40.0))
            })
        );
        assert_eq!(
            parse(&c, "ellipse(50% farthest-side at top 10)"),
            Some(ShapeOutside::Ellipse {
                cx: Dim::Pt(10.0),
                cy: Dim::Pct(0.0),
                rx: Radius::Len(Dim::Pct(50.0)),
                ry: Radius::FarthestSide
            })
        );
        assert_eq!(
            parse(&c, "inset(0 20pt)"),
            Some(ShapeOutside::Inset([
                Dim::Pt(0.0),
                Dim::Pt(20.0),
                Dim::Pt(0.0),
                Dim::Pt(20.0)
            ]))
        );
        assert_eq!(parse(&c, "polygon(0 0, 100% 0)"), None);
        assert_eq!(parse(&c, "url(a.png)"), None);
    }
}
