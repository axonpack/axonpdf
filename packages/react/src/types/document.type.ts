export type Orientation = "portrait" | "landscape";

type StandardPageSize =
  | "4A0"
  | "2A0"
  | "A0"
  | "A1"
  | "A2"
  | "A3"
  | "A4"
  | "A5"
  | "A6"
  | "A7"
  | "A8"
  | "A9"
  | "A10"
  | "B0"
  | "B1"
  | "B2"
  | "B3"
  | "B4"
  | "B5"
  | "B6"
  | "B7"
  | "B8"
  | "B9"
  | "B10"
  | "C0"
  | "C1"
  | "C2"
  | "C3"
  | "C4"
  | "C5"
  | "C6"
  | "C7"
  | "C8"
  | "C9"
  | "C10"
  | "RA0"
  | "RA1"
  | "RA2"
  | "RA3"
  | "RA4"
  | "SRA0"
  | "SRA1"
  | "SRA2"
  | "SRA3"
  | "SRA4"
  | "EXECUTIVE"
  | "FOLIO"
  | "LEGAL"
  | "LETTER"
  | "TABLOID"
  | "ID1";

type StaticSize = number | string;

export type PageSize =
  | StandardPageSize
  | [StaticSize]
  | [StaticSize, StaticSize]
  | { width: StaticSize; height?: StaticSize };

export type PDFVersion = "1.3" | "1.4" | "1.5" | "1.6" | "1.7" | "1.7ext3";

export type PDFConformance =
  | "PDF/A-1"
  | "PDF/A-1b"
  | "PDF/A-2"
  | "PDF/A-2b"
  | "PDF/A-3"
  | "PDF/A-3b";

export type PageLayout =
  | "singlePage"
  | "oneColumn"
  | "twoColumnLeft"
  | "twoColumnRight"
  | "twoPageLeft"
  | "twoPageRight";

export type PageMode =
  | "useNone"
  | "useOutlines"
  | "useThumbs"
  | "fullScreen"
  | "useOC"
  | "useAttachments";

export interface Permissions {
  printing?: "lowResolution" | "highResolution";
  modifying?: boolean;
  copying?: boolean;
  annotating?: boolean;
  fillingForms?: boolean;
  contentAccessibility?: boolean;
  documentAssembly?: boolean;
}

export type Bookmark =
  | string
  | {
      title: string;
      top?: number;
      left?: number;
      zoom?: number;
      fit?: boolean;
      expanded?: boolean;
    };

export type HitSlop = number | { top?: number; bottom?: number; left?: number; right?: number };

type HTTPMethod = "GET" | "HEAD" | "POST" | "PUT" | "DELETE" | "PATCH";

type Source =
  | string
  | Buffer
  | Uint8Array
  | ArrayBuffer
  | Blob
  | { data: Buffer | Uint8Array; format: "png" | "jpg" }
  | {
      uri: string;
      method?: HTTPMethod;
      body?: any;
      headers?: any;
      credentials?: "omit" | "same-origin" | "include";
    }
  | undefined;

export type SourceObject = Source | (() => Source) | Promise<Source> | (() => Promise<Source>);

export type SrcSet = string;

export type Sizes = string | number;

/** Splits a word into parts; the second argument is the built-in split. */
export type HyphenationCallback = (word: string, builtin: (word: string) => string[]) => string[];

export type EmojiSource =
  | { url: string; format?: string; withVariationSelectors?: boolean }
  | { builder: (code: string) => string; withVariationSelectors?: boolean };
