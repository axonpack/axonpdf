//! Colors: CSS names, hex, `rgb()`/`rgba()`, `hsl()`/`hsla()` and `hwb()`. Anything else is black.

use crate::display::Color;

pub fn parse(s: &str) -> Color {
    try_parse(s.trim()).unwrap_or(Color::BLACK)
}

fn try_parse(s: &str) -> Option<Color> {
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    let lower = s.to_ascii_lowercase();
    if let Some((name, args)) = lower.strip_suffix(')').and_then(|x| x.split_once('(')) {
        return parse_fn(name.trim(), args);
    }
    if lower == "transparent" {
        return Some(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        });
    }
    let rgb = NAMED.iter().find(|(n, _)| *n == lower)?.1;
    Some(rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 1.0))
}

fn rgb8(r: u8, g: u8, b: u8, a: f64) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a.clamp(0.0, 1.0) as f32,
    }
}

fn parse_hex(hex: &str) -> Option<Color> {
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let digit = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).unwrap() * 17;
    let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
    match hex.len() {
        3 => Some(rgb8(digit(0), digit(1), digit(2), 1.0)),
        4 => Some(rgb8(digit(0), digit(1), digit(2), digit(3) as f64 / 255.0)),
        6 => Some(rgb8(pair(0), pair(2), pair(4), 1.0)),
        8 => Some(rgb8(pair(0), pair(2), pair(4), pair(6) as f64 / 255.0)),
        _ => None,
    }
}

fn parse_fn(name: &str, args: &str) -> Option<Color> {
    // Commas, spaces and a `/` before alpha all separate arguments.
    let parts: Vec<&str> = args
        .split([',', ' ', '/'])
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let num = |p: &str| -> Option<(f64, bool)> {
        match p.strip_suffix('%') {
            Some(x) => x.parse().ok().map(|n| (n, true)),
            None => p
                .strip_suffix("deg")
                .unwrap_or(p)
                .parse()
                .ok()
                .map(|n| (n, false)),
        }
    };
    let alpha = match parts.get(3) {
        Some(p) => match num(p)? {
            (n, true) => n / 100.0,
            (n, false) => n,
        },
        None => 1.0,
    };
    let channel = |p: &str| -> Option<u8> {
        let (n, pct) = num(p)?;
        let v = if pct { (n * 2.55).round() } else { n.round() };
        Some(v.clamp(0.0, 255.0) as u8)
    };
    match name {
        "rgb" | "rgba" => Some(rgb8(
            channel(parts[0])?,
            channel(parts[1])?,
            channel(parts[2])?,
            alpha,
        )),
        "hsl" | "hsla" | "hwb" => {
            let h = num(parts[0])?.0.rem_euclid(360.0);
            let (x, y) = (
                num(parts[1])?.0.clamp(0.0, 100.0),
                num(parts[2])?.0.clamp(0.0, 100.0),
            );
            let [r, g, b] = if name == "hwb" {
                hwb(h, x, y)
            } else {
                hsl(h, x, y)
            };
            Some(rgb8(r, g, b, alpha))
        }
        _ => None,
    }
}

fn hsl(h: f64, s: f64, l: f64) -> [u8; 3] {
    // Hue, saturation and lightness are whole numbers by the time the color is drawn.
    let (h, s, l) = (h.round(), s.round() / 100.0, l.round() / 100.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r, g, b].map(|v| ((v + m) * 255.0).round() as u8)
}

fn hwb(h: f64, w: f64, bl: f64) -> [u8; 3] {
    let (w, bl) = (w / 100.0, bl / 100.0);
    if w + bl >= 1.0 {
        let gray = (w / (w + bl) * 255.0).round() as u8;
        return [gray; 3];
    }
    hsl(h, 100.0, 50.0)
        .map(|v| ((v as f64 / 255.0) * (1.0 - w - bl) * 255.0 + w * 255.0).round() as u8)
}

/// The CSS named colors.
const NAMED: [(&str, u32); 148] = [
    ("aliceblue", 0xf0f8ff),
    ("antiquewhite", 0xfaebd7),
    ("aqua", 0x00ffff),
    ("aquamarine", 0x7fffd4),
    ("azure", 0xf0ffff),
    ("beige", 0xf5f5dc),
    ("bisque", 0xffe4c4),
    ("black", 0x000000),
    ("blanchedalmond", 0xffebcd),
    ("blue", 0x0000ff),
    ("blueviolet", 0x8a2be2),
    ("brown", 0xa52a2a),
    ("burlywood", 0xdeb887),
    ("cadetblue", 0x5f9ea0),
    ("chartreuse", 0x7fff00),
    ("chocolate", 0xd2691e),
    ("coral", 0xff7f50),
    ("cornflowerblue", 0x6495ed),
    ("cornsilk", 0xfff8dc),
    ("crimson", 0xdc143c),
    ("cyan", 0x00ffff),
    ("darkblue", 0x00008b),
    ("darkcyan", 0x008b8b),
    ("darkgoldenrod", 0xb8860b),
    ("darkgray", 0xa9a9a9),
    ("darkgreen", 0x006400),
    ("darkgrey", 0xa9a9a9),
    ("darkkhaki", 0xbdb76b),
    ("darkmagenta", 0x8b008b),
    ("darkolivegreen", 0x556b2f),
    ("darkorange", 0xff8c00),
    ("darkorchid", 0x9932cc),
    ("darkred", 0x8b0000),
    ("darksalmon", 0xe9967a),
    ("darkseagreen", 0x8fbc8f),
    ("darkslateblue", 0x483d8b),
    ("darkslategray", 0x2f4f4f),
    ("darkslategrey", 0x2f4f4f),
    ("darkturquoise", 0x00ced1),
    ("darkviolet", 0x9400d3),
    ("deeppink", 0xff1493),
    ("deepskyblue", 0x00bfff),
    ("dimgray", 0x696969),
    ("dimgrey", 0x696969),
    ("dodgerblue", 0x1e90ff),
    ("firebrick", 0xb22222),
    ("floralwhite", 0xfffaf0),
    ("forestgreen", 0x228b22),
    ("fuchsia", 0xff00ff),
    ("gainsboro", 0xdcdcdc),
    ("ghostwhite", 0xf8f8ff),
    ("gold", 0xffd700),
    ("goldenrod", 0xdaa520),
    ("gray", 0x808080),
    ("green", 0x008000),
    ("greenyellow", 0xadff2f),
    ("grey", 0x808080),
    ("honeydew", 0xf0fff0),
    ("hotpink", 0xff69b4),
    ("indianred", 0xcd5c5c),
    ("indigo", 0x4b0082),
    ("ivory", 0xfffff0),
    ("khaki", 0xf0e68c),
    ("lavender", 0xe6e6fa),
    ("lavenderblush", 0xfff0f5),
    ("lawngreen", 0x7cfc00),
    ("lemonchiffon", 0xfffacd),
    ("lightblue", 0xadd8e6),
    ("lightcoral", 0xf08080),
    ("lightcyan", 0xe0ffff),
    ("lightgoldenrodyellow", 0xfafad2),
    ("lightgray", 0xd3d3d3),
    ("lightgreen", 0x90ee90),
    ("lightgrey", 0xd3d3d3),
    ("lightpink", 0xffb6c1),
    ("lightsalmon", 0xffa07a),
    ("lightseagreen", 0x20b2aa),
    ("lightskyblue", 0x87cefa),
    ("lightslategray", 0x778899),
    ("lightslategrey", 0x778899),
    ("lightsteelblue", 0xb0c4de),
    ("lightyellow", 0xffffe0),
    ("lime", 0x00ff00),
    ("limegreen", 0x32cd32),
    ("linen", 0xfaf0e6),
    ("magenta", 0xff00ff),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66cdaa),
    ("mediumblue", 0x0000cd),
    ("mediumorchid", 0xba55d3),
    ("mediumpurple", 0x9370db),
    ("mediumseagreen", 0x3cb371),
    ("mediumslateblue", 0x7b68ee),
    ("mediumspringgreen", 0x00fa9a),
    ("mediumturquoise", 0x48d1cc),
    ("mediumvioletred", 0xc71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xf5fffa),
    ("mistyrose", 0xffe4e1),
    ("moccasin", 0xffe4b5),
    ("navajowhite", 0xffdead),
    ("navy", 0x000080),
    ("oldlace", 0xfdf5e6),
    ("olive", 0x808000),
    ("olivedrab", 0x6b8e23),
    ("orange", 0xffa500),
    ("orangered", 0xff4500),
    ("orchid", 0xda70d6),
    ("palegoldenrod", 0xeee8aa),
    ("palegreen", 0x98fb98),
    ("paleturquoise", 0xafeeee),
    ("palevioletred", 0xdb7093),
    ("papayawhip", 0xffefd5),
    ("peachpuff", 0xffdab9),
    ("peru", 0xcd853f),
    ("pink", 0xffc0cb),
    ("plum", 0xdda0dd),
    ("powderblue", 0xb0e0e6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("red", 0xff0000),
    ("rosybrown", 0xbc8f8f),
    ("royalblue", 0x4169e1),
    ("saddlebrown", 0x8b4513),
    ("salmon", 0xfa8072),
    ("sandybrown", 0xf4a460),
    ("seagreen", 0x2e8b57),
    ("seashell", 0xfff5ee),
    ("sienna", 0xa0522d),
    ("silver", 0xc0c0c0),
    ("skyblue", 0x87ceeb),
    ("slateblue", 0x6a5acd),
    ("slategray", 0x708090),
    ("slategrey", 0x708090),
    ("snow", 0xfffafa),
    ("springgreen", 0x00ff7f),
    ("steelblue", 0x4682b4),
    ("tan", 0xd2b48c),
    ("teal", 0x008080),
    ("thistle", 0xd8bfd8),
    ("tomato", 0xff6347),
    ("turquoise", 0x40e0d0),
    ("violet", 0xee82ee),
    ("wheat", 0xf5deb3),
    ("white", 0xffffff),
    ("whitesmoke", 0xf5f5f5),
    ("yellow", 0xffff00),
    ("yellowgreen", 0x9acd32),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(c: Color) -> [u8; 4] {
        [c.r, c.g, c.b, c.a].map(|v| (v * 255.0).round() as u8)
    }

    #[test]
    fn colors() {
        assert_eq!(rgba(parse("red")), [255, 0, 0, 255]);
        assert_eq!(rgba(parse("RebeccaPurple")), [102, 51, 153, 255]);
        assert_eq!(rgba(parse("#0f0")), [0, 255, 0, 255]);
        assert_eq!(rgba(parse("#11223380")), [17, 34, 51, 128]);
        assert_eq!(rgba(parse("rgb(10, 20, 30)")), [10, 20, 30, 255]);
        assert_eq!(rgba(parse("rgba(10,20,30,0.5)")), [10, 20, 30, 128]);
        assert_eq!(rgba(parse("hsl(120, 100%, 50%)")), [0, 255, 0, 255]);
        assert_eq!(rgba(parse("transparent")), [0, 0, 0, 0]);
        assert_eq!(rgba(parse("nonsense")), [0, 0, 0, 255]);
    }
}
