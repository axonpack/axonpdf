// The types a document's code names. Styles and the document's own types are defined here;
// @axonpdf/react and @axonpdf/stylesheet read them from this package.
export type * from "./document.type.js";
export type * from "./style.type.js";
export type * from "./svg.type.js";
export type { FontDescriptor, FontStoreType as FontStore } from "@axonpdf/font";

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
