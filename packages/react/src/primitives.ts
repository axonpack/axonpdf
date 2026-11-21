import type { ComponentType, ReactNode } from "react";

import type {
  CheckboxProps,
  FieldSetProps,
  SelectAndListProps,
  TextInputProps,
} from "./types/form-props.type.js";
import type {
  CanvasProps,
  DocumentProps,
  ImageBackgroundProps,
  ImageProps,
  LinkProps,
  NoteProps,
  PageProps,
  TextProps,
  ViewProps,
} from "./types/props.type.js";
import type {
  CircleProps,
  ClipPathProps,
  DefsProps,
  EllipseProps,
  GProps,
  LinearGradientProps,
  LineProps,
  MarkerProps,
  PathProps,
  PolygonProps,
  PolylineProps,
  RadialGradientProps,
  RectProps,
  StopProps,
  SVGProps,
  SVGTextProps,
  TspanProps,
} from "./types/svg-props.type.js";

// The components are host element names: the reconciler turns `<View>` into a `VIEW` node.

export const Document = "DOCUMENT" as unknown as ComponentType<DocumentProps>;
export const Page = "PAGE" as unknown as ComponentType<PageProps>;
export const View = "VIEW" as unknown as ComponentType<ViewProps>;
export const Text = "TEXT" as unknown as ComponentType<TextProps | SVGTextProps>;
export const Link = "LINK" as unknown as ComponentType<LinkProps>;
export const Image = "IMAGE" as unknown as ComponentType<ImageProps>;
export const ImageBackground = "IMAGE_BACKGROUND" as unknown as ComponentType<
  ImageBackgroundProps & { children?: ReactNode }
>;
export const Note = "NOTE" as unknown as ComponentType<NoteProps>;
export const Canvas = "CANVAS" as unknown as ComponentType<CanvasProps>;

export const Svg = "SVG" as unknown as ComponentType<SVGProps>;
export const G = "G" as unknown as ComponentType<GProps>;
export const Path = "PATH" as unknown as ComponentType<PathProps>;
export const Rect = "RECT" as unknown as ComponentType<RectProps>;
export const Line = "LINE" as unknown as ComponentType<LineProps>;
export const Circle = "CIRCLE" as unknown as ComponentType<CircleProps>;
export const Ellipse = "ELLIPSE" as unknown as ComponentType<EllipseProps>;
export const Polygon = "POLYGON" as unknown as ComponentType<PolygonProps>;
export const Polyline = "POLYLINE" as unknown as ComponentType<PolylineProps>;
export const Tspan = "TSPAN" as unknown as ComponentType<TspanProps>;
export const Defs = "DEFS" as unknown as ComponentType<DefsProps>;
export const Stop = "STOP" as unknown as ComponentType<StopProps>;
export const ClipPath = "CLIP_PATH" as unknown as ComponentType<ClipPathProps>;
export const LinearGradient = "LINEAR_GRADIENT" as unknown as ComponentType<LinearGradientProps>;
export const RadialGradient = "RADIAL_GRADIENT" as unknown as ComponentType<RadialGradientProps>;
export const Marker = "MARKER" as unknown as ComponentType<MarkerProps>;

export const FieldSet = "FIELD_SET" as unknown as ComponentType<FieldSetProps>;
export const TextInput = "TEXT_INPUT" as unknown as ComponentType<TextInputProps>;
export const Select = "SELECT" as unknown as ComponentType<SelectAndListProps>;
export const Checkbox = "CHECKBOX" as unknown as ComponentType<CheckboxProps>;
export const List = "LIST" as unknown as ComponentType<SelectAndListProps>;

export const TextInstance = "TEXT_INSTANCE";
export const Fragment = "FRAGMENT";
