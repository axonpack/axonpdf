export type FontStyle = "normal" | "italic" | "oblique";

export type FontWeight =
  | number
  | "thin"
  | "hairline"
  | "ultralight"
  | "extralight"
  | "light"
  | "normal"
  | "medium"
  | "semibold"
  | "demibold"
  | "bold"
  | "ultrabold"
  | "extrabold"
  | "heavy"
  | "black";

export interface FontDescriptor {
  fontFamily: string;
  fontStyle?: FontStyle;
  fontWeight?: FontWeight;
}

export interface RemoteOptions {
  method?: "GET" | "HEAD" | "POST" | "PUT" | "DELETE" | "PATCH";
  headers?: Record<string, string>;
  body?: any;
}

export type FontSourceOptions = { postscriptName?: string } & RemoteOptions;

/** A font as registered: a URL, data URL, file path, or a standard font's name. */
export type FontInput = {
  src: string;
  fontStyle?: FontStyle;
  fontWeight?: FontWeight;
} & FontSourceOptions;

export type SingleLoad = { family: string } & FontInput;

export type BulkLoad = { family: string; fonts: FontInput[] };

/** A registered font, with its bytes once loaded. */
export interface FontSource extends FontSourceOptions {
  src: string;
  family: string;
  fontStyle: FontStyle;
  fontWeight: number;
  data: Uint8Array | null;
}

export interface FontFamily {
  family: string;
  sources: FontSource[];
}

export type EmojiSource =
  | { url: string; format?: string; withVariationSelectors?: boolean }
  | { builder: (code: string) => string; withVariationSelectors?: boolean };

/** Splits a word into parts; the second argument is the built-in split. */
export type HyphenationCallback = (word: string, builtin: (word: string) => string[]) => string[];
