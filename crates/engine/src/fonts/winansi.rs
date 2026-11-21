//! WinAnsiEncoding: the one-byte encoding the Standard 14 fonts are written with.

/// The byte a character is written as. Characters past 255 that the encoding lacks give their
/// low code unchanged, which names `.notdef` (zero width).
pub fn code(c: char) -> u16 {
    let u = c as u32;
    let mapped = match u {
        0x0192 => 131,
        0x2013 => 150,
        0x2014 => 151,
        0x2018 => 145,
        0x2019 => 146,
        0x201A => 130,
        0x201C => 147,
        0x201D => 148,
        0x201E => 132,
        0x2020 => 134,
        0x2021 => 135,
        0x2022 => 149,
        0x2026 => 133,
        0x20AC => 128,
        0x2030 => 137,
        0x2039 => 139,
        0x203A => 155,
        0x02C6 => 136,
        0x2122 => 153,
        0x0152 => 140,
        0x0153 => 156,
        0x02DC => 152,
        0x0160 => 138,
        0x0161 => 154,
        0x0178 => 159,
        0x017D => 142,
        0x017E => 158,
        _ => u,
    };
    mapped.min(u16::MAX as u32) as u16
}

/// The glyph name behind a code; `.notdef` past 255.
pub fn name(code: u16) -> &'static str {
    NAMES.get(code as usize).copied().unwrap_or(".notdef")
}

/// Glyph names by code. Code 159 names `ydieresis`, not `Ydieresis`, and 0x80 to 0x9F also
/// answer for the C1 control characters; both keep the widths documents were laid out with.
#[rustfmt::skip]
pub const NAMES: [&str; 256] = [
    ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef",
    ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef",
    ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef",
    ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef", ".notdef",
    "space", "exclam", "quotedbl", "numbersign", "dollar", "percent", "ampersand", "quotesingle",
    "parenleft", "parenright", "asterisk", "plus", "comma", "hyphen", "period", "slash",
    "zero", "one", "two", "three", "four", "five", "six", "seven",
    "eight", "nine", "colon", "semicolon", "less", "equal", "greater", "question",
    "at", "A", "B", "C", "D", "E", "F", "G",
    "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W",
    "X", "Y", "Z", "bracketleft", "backslash", "bracketright", "asciicircum", "underscore",
    "grave", "a", "b", "c", "d", "e", "f", "g",
    "h", "i", "j", "k", "l", "m", "n", "o",
    "p", "q", "r", "s", "t", "u", "v", "w",
    "x", "y", "z", "braceleft", "bar", "braceright", "asciitilde", ".notdef",
    "Euro", ".notdef", "quotesinglbase", "florin", "quotedblbase", "ellipsis", "dagger", "daggerdbl",
    "circumflex", "perthousand", "Scaron", "guilsinglleft", "OE", ".notdef", "Zcaron", ".notdef",
    ".notdef", "quoteleft", "quoteright", "quotedblleft", "quotedblright", "bullet", "endash", "emdash",
    "tilde", "trademark", "scaron", "guilsinglright", "oe", ".notdef", "zcaron", "ydieresis",
    "space", "exclamdown", "cent", "sterling", "currency", "yen", "brokenbar", "section",
    "dieresis", "copyright", "ordfeminine", "guillemotleft", "logicalnot", "hyphen", "registered", "macron",
    "degree", "plusminus", "twosuperior", "threesuperior", "acute", "mu", "paragraph", "periodcentered",
    "cedilla", "onesuperior", "ordmasculine", "guillemotright", "onequarter", "onehalf", "threequarters", "questiondown",
    "Agrave", "Aacute", "Acircumflex", "Atilde", "Adieresis", "Aring", "AE", "Ccedilla",
    "Egrave", "Eacute", "Ecircumflex", "Edieresis", "Igrave", "Iacute", "Icircumflex", "Idieresis",
    "Eth", "Ntilde", "Ograve", "Oacute", "Ocircumflex", "Otilde", "Odieresis", "multiply",
    "Oslash", "Ugrave", "Uacute", "Ucircumflex", "Udieresis", "Yacute", "Thorn", "germandbls",
    "agrave", "aacute", "acircumflex", "atilde", "adieresis", "aring", "ae", "ccedilla",
    "egrave", "eacute", "ecircumflex", "edieresis", "igrave", "iacute", "icircumflex", "idieresis",
    "eth", "ntilde", "ograve", "oacute", "ocircumflex", "otilde", "odieresis", "divide",
    "oslash", "ugrave", "uacute", "ucircumflex", "udieresis", "yacute", "thorn", "ydieresis",
];
