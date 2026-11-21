// The types a document's code names, from the packages that define them.
export type { Style, StyleProp } from "@axonpdf/stylesheet";
export type {
  Bookmark,
  HitSlop,
  Orientation,
  PageLayout,
  PageMode,
  PageSize,
  PDFConformance,
  PDFVersion,
  Permissions,
  SVGPresentationAttributes,
  Sizes,
  SourceObject,
  SrcSet,
} from "@axonpdf/react";
export type {
  EmojiSource,
  FontDescriptor,
  FontStoreType as FontStore,
  FontStyle,
  FontWeight,
  HyphenationCallback,
} from "@axonpdf/font";

/** The node types a document is made of, as the components name them. */
export enum Primitive {
  G = "G",
  Svg = "SVG",
  View = "VIEW",
  Text = "TEXT",
  Link = "LINK",
  Page = "PAGE",
  Note = "NOTE",
  Path = "PATH",
  Rect = "RECT",
  Line = "LINE",
  Stop = "STOP",
  Defs = "DEFS",
  Image = "IMAGE",
  Tspan = "TSPAN",
  Canvas = "CANVAS",
  Circle = "CIRCLE",
  Ellipse = "ELLIPSE",
  Polygon = "POLYGON",
  Document = "DOCUMENT",
  Polyline = "POLYLINE",
  ClipPath = "CLIP_PATH",
  TextInstance = "TEXT_INSTANCE",
  Fragment = "FRAGMENT",
  LinearGradient = "LINEAR_GRADIENT",
  RadialGradient = "RADIAL_GRADIENT",
}
