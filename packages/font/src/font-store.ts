import type {
  BulkLoad,
  EmojiSource,
  FontDescriptor,
  FontFamily,
  FontSource,
  FontWeight,
  HyphenationCallback,
  RemoteOptions,
  SingleLoad,
} from "./types.js";

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

/** Fetches a font's bytes; `@axonpdf/react` gives the shared store one that also reads files. */
export type FontLoader = (src: string, options: RemoteOptions) => Promise<Uint8Array>;

const fetchFont: FontLoader = async (src, options) => {
  const response = await fetch(src, options);
  if (!response.ok) throw new Error(`Failed to fetch ${src}: ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
};

/** Registered fonts by family, with the emoji source and the hyphenation callback. */
export default class FontStore {
  fontFamilies: Record<string, FontFamily> = {};
  emojiSource: EmojiSource | null = null;
  hyphenationCallback: HyphenationCallback | null = null;
  loader: FontLoader = fetchFont;

  constructor() {
    for (const [family, [regular, bold, italic, boldItalic]] of Object.entries(STANDARD)) {
      this.register({
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

  register = (data: SingleLoad | BulkLoad) => {
    const { family } = data;
    this.fontFamilies[family] ??= { family, sources: [] };
    const fonts = "fonts" in data ? data.fonts : [data];
    for (const { src, fontStyle = "normal", fontWeight, ...options } of fonts) {
      const { family: _, ...rest } = options as typeof options & { family?: string };
      this.fontFamilies[family].sources.push({
        ...rest,
        src,
        family,
        fontStyle,
        fontWeight: fontWeightNumber(fontWeight),
        data: null,
      });
    }
  };

  registerEmojiSource = (source: EmojiSource) => {
    this.emojiSource = source;
  };

  registerHyphenationCallback = (callback: HyphenationCallback) => {
    this.hyphenationCallback = callback;
  };

  getFont = ({ fontFamily, fontStyle = "normal", fontWeight }: FontDescriptor) => {
    const family = this.fontFamilies[fontFamily];
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
  };

  /** Loads a registered font's bytes; standard fonts have none. */
  loadSource = async (source: FontSource): Promise<Uint8Array | null> => {
    if (isStandardFont(source.src)) return null;
    const { method, headers, body } = source;
    source.data ??= await this.loader(source.src, { method, headers, body });
    return source.data;
  };

  load = async (descriptor: FontDescriptor) => {
    const source = this.getFont(descriptor);
    if (source) await this.loadSource(source);
  };

  reset = () => {
    for (const family of Object.values(this.fontFamilies)) {
      for (const source of family.sources) source.data = null;
    }
  };

  clear = () => {
    this.fontFamilies = {};
    this.emojiSource = null;
    this.hyphenationCallback = null;
  };

  getRegisteredFonts = () => this.fontFamilies;
  getRegisteredFontFamilies = () => Object.keys(this.fontFamilies);
  getEmojiSource = () => this.emojiSource;
  getHyphenationCallback = () => this.hyphenationCallback;
}

export type FontStoreType = FontStore;
