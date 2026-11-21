import type { FontStyle, FontWeight } from "@axonpdf/font";

export type { FontStyle, FontWeight };

type Size = number | string;
type Percentage = `${string}%`;

export type BorderStyleValue = "dashed" | "dotted" | "solid";
export type Transform =
  | { operation: "scale"; value: [number, number] }
  | { operation: "translate"; value: [number, number] }
  | { operation: "rotate"; value: [number, number, number] }
  | { operation: "skew"; value: [number, number] }
  | { operation: "matrix"; value: [number, number, number, number, number, number] };

type ShapeScalar = number | Percentage;
export type ShapeOutside =
  | {
      type: "circle";
      cx: ShapeScalar;
      cy: ShapeScalar;
      r: ShapeScalar | "closest-side" | "farthest-side";
    }
  | { type: "ellipse"; cx: ShapeScalar; cy: ShapeScalar; rx: ShapeScalar; ry: ShapeScalar }
  | { type: "polygon"; points: [ShapeScalar, ShapeScalar][] }
  | { type: "inset"; top: ShapeScalar; right: ShapeScalar; bottom: ShapeScalar; left: ShapeScalar };

/** Every style key the engine reads. Values travel to the engine exactly as written. */
export interface BaseStyle {
  // Flexbox and layout
  display?: "flex" | "none";
  flex?: Size;
  flexDirection?: "row" | "row-reverse" | "column" | "column-reverse";
  flexWrap?: "wrap" | "wrap-reverse" | "nowrap";
  flexFlow?: Size;
  flexGrow?: Size;
  flexShrink?: Size;
  flexBasis?: Size;
  alignContent?:
    | "flex-start"
    | "flex-end"
    | "center"
    | "stretch"
    | "space-between"
    | "space-around"
    | "space-evenly";
  alignItems?: "flex-start" | "flex-end" | "center" | "stretch" | "baseline";
  alignSelf?: "auto" | "flex-start" | "flex-end" | "center" | "baseline" | "stretch";
  justifyContent?:
    | "flex-start"
    | "flex-end"
    | "center"
    | "space-around"
    | "space-between"
    | "space-evenly";
  justifySelf?: string;
  gap?: Size;
  rowGap?: Size;
  columnGap?: Size;
  aspectRatio?: Size;
  position?: "absolute" | "relative" | "static";
  top?: Size;
  right?: Size;
  bottom?: Size;
  left?: Size;
  zIndex?: Size;
  overflow?: "hidden";
  float?: "left" | "right" | "none";
  clear?: "left" | "right" | "both" | "none";
  shapeOutside?: string | ShapeOutside;
  // Dimensions
  width?: Size;
  height?: Size;
  minWidth?: Size;
  minHeight?: Size;
  maxWidth?: Size;
  maxHeight?: Size;
  // Margins and paddings
  margin?: Size;
  marginHorizontal?: Size;
  marginVertical?: Size;
  marginTop?: Size;
  marginRight?: Size;
  marginBottom?: Size;
  marginLeft?: Size;
  padding?: Size;
  paddingHorizontal?: Size;
  paddingVertical?: Size;
  paddingTop?: Size;
  paddingRight?: Size;
  paddingBottom?: Size;
  paddingLeft?: Size;
  // Borders
  border?: Size;
  borderTop?: Size;
  borderRight?: Size;
  borderBottom?: Size;
  borderLeft?: Size;
  borderColor?: string;
  borderRadius?: Size;
  borderStyle?: BorderStyleValue;
  borderWidth?: Size;
  borderTopColor?: string;
  borderTopStyle?: BorderStyleValue;
  borderTopWidth?: Size;
  borderRightColor?: string;
  borderRightStyle?: BorderStyleValue;
  borderRightWidth?: Size;
  borderBottomColor?: string;
  borderBottomStyle?: BorderStyleValue;
  borderBottomWidth?: Size;
  borderLeftColor?: string;
  borderLeftStyle?: BorderStyleValue;
  borderLeftWidth?: Size;
  borderTopLeftRadius?: Size;
  borderTopRightRadius?: Size;
  borderBottomRightRadius?: Size;
  borderBottomLeftRadius?: Size;
  // Color
  color?: string;
  backgroundColor?: string;
  opacity?: Size;
  // Text
  direction?: "ltr" | "rtl";
  fontFamily?: string | string[];
  fontSize?: Size;
  fontStyle?: FontStyle;
  fontWeight?: FontWeight;
  fontFeatureSettings?: string[] | Record<string, number>;
  letterSpacing?: Size;
  lineHeight?: Size;
  maxLines?: Size;
  textAlign?: "left" | "right" | "center" | "justify";
  textDecoration?:
    | "line-through"
    | "underline"
    | "none"
    | "line-through underline"
    | "underline line-through";
  textDecorationColor?: string;
  textDecorationStyle?: "dashed" | "dotted" | "solid" | string;
  textIndent?: Size;
  textOverflow?: "ellipsis";
  textTransform?: "capitalize" | "lowercase" | "uppercase" | "upperfirst" | "none";
  verticalAlign?: "sub" | "super";
  wordSpacing?: Size;
  // Transforms and images
  transform?: string | Transform[];
  transformOrigin?: Size;
  transformOriginX?: Size;
  transformOriginY?: Size;
  gradientTransform?: string | Transform[];
  objectFit?: string;
  objectPosition?: Size;
  objectPositionX?: Size;
  objectPositionY?: Size;
  // SVG paint
  fill?: string;
  stroke?: string;
  strokeDasharray?: string;
  strokeWidth?: Size;
  fillOpacity?: Size;
  fillRule?: "nonzero" | "evenodd";
  strokeOpacity?: Size;
  textAnchor?: "start" | "middle" | "end";
  strokeLinecap?: "butt" | "round" | "square";
  strokeLinejoin?: "butt" | "round" | "square" | "miter" | "bevel";
  visibility?: "visible" | "hidden" | "collapse";
  clipPath?: string;
  dominantBaseline?:
    | "auto"
    | "middle"
    | "central"
    | "hanging"
    | "mathematical"
    | "text-after-edge"
    | "text-before-edge";
}

/** Media queries against the page box, such as `"@media max-width: 400"`. */
export type MediaQueryStyle = { [key: `@media${string}`]: BaseStyle };

export type Style = BaseStyle & MediaQueryStyle;

type RecursiveArray<T> = (T | readonly T[] | RecursiveArray<T>)[];

export type StyleProp =
  | Style
  | Style[]
  | RecursiveArray<Style | Style[] | undefined | null | false>;

export interface Styles {
  [key: string]: Style;
}
