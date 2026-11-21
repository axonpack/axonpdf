import type { ReactElement, ReactNode } from "react";

import type {
  Bookmark,
  HitSlop,
  HyphenationCallback,
  Orientation,
  PageLayout,
  PageMode,
  PageSize,
  PDFConformance,
  PDFVersion,
  Permissions,
  Sizes,
  SourceObject,
  SrcSet,
} from "./document.type.js";
import type { Style, StyleProp } from "./style.type.js";

export interface OnRenderProps {
  blob?: Blob;
}

export interface DocumentProps {
  style?: StyleProp;
  title?: string;
  author?: string;
  subject?: string;
  creator?: string;
  keywords?: string;
  producer?: string;
  language?: string;
  creationDate?: Date;
  modificationDate?: Date;
  pdfVersion?: PDFVersion;
  conformance?: PDFConformance;
  pageMode?: PageMode;
  pageLayout?: PageLayout;
  ownerPassword?: string;
  userPassword?: string;
  permissions?: Permissions;
  onRender?: (props: OnRenderProps) => any;
  children?: ReactNode;
}

/** Props every primitive takes. `breakBefore`, `breakAfter` and `breakInside` are axonpdf's. */
export interface NodeProps {
  id?: string;
  style?: StyleProp;
  /** Repeats the element on every page its `Page` wraps into. */
  fixed?: boolean;
  /** Starts a new page at the element. */
  break?: boolean;
  /** No page break within this many points after the element. */
  minPresenceAhead?: number;
  /** `'page'` is the same as `break`. */
  breakBefore?: "page" | "auto";
  /** `'page'` starts a new page after the element. */
  breakAfter?: "page" | "auto";
  /** `'avoid'` is the same as `wrap={false}`. */
  breakInside?: "avoid" | "auto";
  bookmark?: Bookmark;
  children?: ReactNode;
}

export interface PageLayoutProps {
  children: ReactNode;
  pageNumber?: number;
  totalPages?: number;
  subPageNumber?: number;
  subPageTotalPages?: number;
}

/** What a `Page` header or footer function receives. */
export interface PageBandInfo {
  pageNumber: number;
  totalPages: number;
  isFirst: boolean;
  isLast: boolean;
  isOdd: boolean;
}

export type PageBand = ReactElement | ((info: PageBandInfo) => ReactNode);

export interface PageProps extends NodeProps {
  wrap?: boolean;
  layout?: (props: PageLayoutProps) => ReactNode;
  experimentalPagination?: boolean;
  debug?: boolean;
  size?: PageSize;
  orientation?: Orientation;
  dpi?: number;
  /** A band at the top of every page that shrinks the content area. */
  header?: PageBand;
  /** A band at the bottom of every page that shrinks the content area. */
  footer?: PageBand;
}

export interface ViewProps extends NodeProps {
  wrap?: boolean;
  debug?: boolean;
  render?: (props: { pageNumber: number; subPageNumber: number }) => ReactNode;
}

export interface TextProps extends NodeProps {
  wrap?: boolean;
  debug?: boolean;
  render?: (props: {
    pageNumber: number;
    totalPages: number;
    subPageNumber: number;
    subPageTotalPages: number;
  }) => ReactNode;
  hyphenationCallback?: HyphenationCallback;
  hyphenationPenalty?: number;
  orphans?: number;
  widows?: number;
}

export interface LinkProps extends NodeProps {
  wrap?: boolean;
  debug?: boolean;
  href?: string;
  src?: string;
  hitSlop?: HitSlop;
}

interface BaseImageProps extends NodeProps {
  debug?: boolean;
  cache?: boolean;
  srcSet?: SrcSet;
  sizes?: Sizes;
}

export type ImageProps = BaseImageProps & ({ src: SourceObject } | { source: SourceObject });

export type ImageBackgroundProps = BaseImageProps & {
  imageStyle?: Style;
} & ({ src: SourceObject } | { source: SourceObject });

export interface NoteProps extends NodeProps {
  children: string;
}

export interface CanvasProps extends NodeProps {
  debug?: boolean;
  paint: (painter: any, availableWidth: number, availableHeight: number) => null;
}

// axonpdf's own components

export interface RowsProps<T> {
  /** An array, or an async iterable that yields rows as they arrive. */
  rows: Iterable<T> | AsyncIterable<T>;
  /** How many rows are laid out and written at a time. Default 100. */
  batchSize?: number;
  children: (row: T, index: number) => ReactNode;
}

export interface PageNumberProps {
  style?: StyleProp;
}

export interface TableProps extends NodeProps {
  wrap?: boolean;
}

export interface TableRowProps extends NodeProps {
  wrap?: boolean;
}
