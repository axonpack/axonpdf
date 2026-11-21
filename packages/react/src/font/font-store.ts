import { loadFontSource } from "./load-fonts.js";
import type { EmojiSource, HyphenationCallback } from "../types/document.type.js";
import type { FontStyle, FontWeight } from "../types/style.type.js";

export interface FontSource {
  src: string;
  family: string;
  fontStyle: FontStyle;
  fontWeight: number;
  postscriptName?: string;
  method?: string;
  headers?: Record<string, string>;
  body?: any;
  /** The font bytes once loaded. */
  data: Uint8Array | null;
}

export interface FontDescriptor {
  fontFamily: string;
  fontStyle?: FontStyle;
  fontWeight?: FontWeight;
}

type SourceInput = Omit<FontSource, "family" | "fontStyle" | "fontWeight" | "data"> & {
  fontStyle?: FontStyle;
  fontWeight?: FontWeight;
};
type SingleLoad = { family: string } & SourceInput;
type BulkLoad = { family: string; fonts: SourceInput[] };

const WEIGHTS: Record<string, number> = {
  thin: 100,
  hairline: 100,
  ultralight: 200,
  extralight: 200,
  light: 300,
  normal: 400,
  medium: 500,
  semibold: 600,
  demibold: 600,
  bold: 700,
  ultrabold: 800,
  extrabold: 800,
  heavy: 900,
  black: 900,
};

export const fontWeightNumber = (weight: FontWeight = 400) =>
  typeof weight === "string" ? (WEIGHTS[weight] ?? 400) : weight;

/** The PDF Standard 14 families: built into every viewer, drawn from metrics, never embedded. */
const STANDARD: Record<string, string[]> = {
  Helvetica: ["Helvetica", "Helvetica-Bold", "Helvetica-Oblique", "Helvetica-BoldOblique"],
  Courier: ["Courier", "Courier-Bold", "Courier-Oblique", "Courier-BoldOblique"],
  "Times-Roman": ["Times-Roman", "Times-Bold", "Times-Italic", "Times-BoldItalic"],
};
export const isStandardFont = (src: string) => Object.values(STANDARD).flat().includes(src);

let families: Record<string, { family: string; sources: FontSource[] }> = {};
let emojiSource: EmojiSource | null = null;
let hyphenationCallback: HyphenationCallback | null = null;

function register(data: SingleLoad | BulkLoad) {
  const { family } = data;
  families[family] ??= { family, sources: [] };
  const fonts = "fonts" in data ? data.fonts : [data];
  for (const { src, fontStyle = "normal", fontWeight, ...options } of fonts) {
    const { family: _, ...rest } = options as SourceInput & { family?: string };
    families[family].sources.push({
      ...rest,
      src,
      family,
      fontStyle,
      fontWeight: fontWeightNumber(fontWeight),
      data: null,
    });
  }
}

function registerStandardFonts() {
  for (const [family, [regular, bold, italic, boldItalic]] of Object.entries(STANDARD)) {
    register({
      family,
      fonts: [
        { src: regular, fontWeight: 400 },
        { src: bold, fontWeight: 700 },
        { src: italic, fontStyle: "italic", fontWeight: 400 },
        { src: boldItalic, fontStyle: "italic", fontWeight: 700 },
      ],
    });
  }
}
registerStandardFonts();

function getFont({ fontFamily, fontStyle = "normal", fontWeight }: FontDescriptor) {
  const family = families[fontFamily];
  if (!family) {
    throw new Error(
      `Font family not registered: ${fontFamily}. Please register it calling Font.register() method.`,
    );
  }
  const weight = fontWeightNumber(fontWeight);
  const sources = family.sources.filter((s) => s.fontStyle === fontStyle);
  // ponytail: nearest weight by distance; the engine does the full CSS fallback order at render.
  return sources.sort(
    (a, b) => Math.abs(a.fontWeight - weight) - Math.abs(b.fontWeight - weight),
  )[0];
}

export const Font = {
  register,
  registerHyphenationCallback: (callback: HyphenationCallback) => {
    hyphenationCallback = callback;
  },
  registerEmojiSource: (source: EmojiSource) => {
    emojiSource = source;
  },
  getFont,
  load: async (descriptor: FontDescriptor) => {
    const source = getFont(descriptor);
    if (source) await loadFontSource(source);
  },
  reset: () => {
    for (const family of Object.values(families)) for (const s of family.sources) s.data = null;
  },
  clear: () => {
    families = {};
    emojiSource = null;
    hyphenationCallback = null;
  },
  getRegisteredFonts: () => families,
  getRegisteredFontFamilies: () => Object.keys(families),
  getEmojiSource: () => emojiSource,
  getHyphenationCallback: () => hyphenationCallback,
};
