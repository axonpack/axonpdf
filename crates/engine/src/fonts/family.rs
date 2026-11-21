//! Picking a face from a family by weight and style (the CSS fallback-weight rules).

use crate::display::FontId;

#[derive(Clone, Copy, Debug)]
pub struct Source {
    pub weight: u16,
    pub italic: bool,
    pub font: FontId,
}

/// Exact weight first. Asked 400 to 500: the first face registered from it up to 500, else the
/// last one at or below it, else the first above 500. Below 400: the heaviest lighter one, else
/// the lightest heavier one. Above 500: the reverse. Only faces of the asked style count.
pub fn pick(sources: &[Source], weight: u16, italic: bool) -> Option<FontId> {
    let style: Vec<&Source> = sources.iter().filter(|s| s.italic == italic).collect();
    if let Some(s) = style.iter().find(|s| s.weight == weight) {
        return Some(s.font);
    }
    // On ties max_by_key keeps the last and min_by_key the first, as a stable sort would.
    let lighter = || {
        style
            .iter()
            .filter(|s| s.weight < weight)
            .max_by_key(|s| s.weight)
    };
    let heavier = || {
        style
            .iter()
            .filter(|s| s.weight > weight)
            .min_by_key(|s| s.weight)
    };
    let found = if (400..=500).contains(&weight) {
        style
            .iter()
            .find(|s| s.weight >= weight && s.weight <= 500)
            .or_else(|| style.iter().rev().find(|s| s.weight <= weight))
            .or_else(|| style.iter().find(|s| s.weight > 500))
    } else if weight < 400 {
        lighter().or_else(heavier)
    } else {
        heavier().or_else(lighter)
    };
    found.map(|s| s.font)
}

/// Named weights (`bold`, `light`, ...) as numbers.
pub fn weight_from_name(name: &str) -> Option<u16> {
    Some(match name {
        "thin" | "hairline" => 100,
        "ultralight" | "extralight" => 200,
        "light" => 300,
        "normal" => 400,
        "medium" => 500,
        "semibold" | "demibold" => 600,
        "bold" => 700,
        "ultrabold" | "extrabold" => 800,
        "heavy" | "black" => 900,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(weight: u16, n: u32) -> Source {
        Source {
            weight,
            italic: false,
            font: FontId(n),
        }
    }

    #[test]
    fn falls_back_by_css_rules() {
        let set = [src(300, 0), src(400, 1), src(700, 2), src(900, 3)];
        assert_eq!(pick(&set, 400, false), Some(FontId(1)));
        assert_eq!(pick(&set, 500, false), Some(FontId(1)));
        assert_eq!(pick(&set, 600, false), Some(FontId(2)));
        assert_eq!(pick(&set, 800, false), Some(FontId(3)));
        assert_eq!(pick(&set, 200, false), Some(FontId(0)));
        assert_eq!(pick(&set[2..], 450, false), Some(FontId(2)));
        assert_eq!(pick(&set, 400, true), None);
    }
}
