import type { ReactNode } from "react";

import type { HyphenationCallback } from "./document.type.js";
import type { NodeProps } from "./props.type.js";
import type { Style } from "./style.type.js";

type Length = string | number;

export interface SVGPresentationAttributes {
  fill?: string;
  color?: string;
  stroke?: string;
  transform?: string;
  strokeDasharray?: string;
  opacity?: Length;
  strokeWidth?: Length;
  fillOpacity?: Length;
  fillRule?: "nonzero" | "evenodd";
  strokeOpacity?: Length;
  textAnchor?: "start" | "middle" | "end";
  strokeLinecap?: "butt" | "round" | "square";
  strokeLinejoin?: "butt" | "round" | "square" | "miter" | "bevel";
  visibility?: "visible" | "hidden" | "collapse";
  clipPath?: string;
  markerStart?: string;
  markerMid?: string;
  markerEnd?: string;
  dominantBaseline?:
    | "auto"
    | "middle"
    | "central"
    | "hanging"
    | "mathematical"
    | "text-after-edge"
    | "text-before-edge";
}

interface Shape extends SVGPresentationAttributes {
  style?: SVGPresentationAttributes;
  children?: ReactNode;
}

export interface SVGProps extends NodeProps, SVGPresentationAttributes {
  debug?: boolean;
  width?: Length;
  height?: Length;
  viewBox?: string;
  preserveAspectRatio?: string;
}

export interface SVGTextProps extends Shape {
  x: Length;
  y: Length;
  hyphenationCallback?: HyphenationCallback;
}

export interface LineProps extends Shape {
  x1: Length;
  x2: Length;
  y1: Length;
  y2: Length;
}

export interface PolylineProps extends Shape {
  points: string;
}

export type PolygonProps = PolylineProps;

export interface PathProps extends Shape {
  d: string;
}

export interface RectProps extends Shape {
  x?: Length;
  y?: Length;
  width: Length;
  height: Length;
  rx?: Length;
  ry?: Length;
}

export interface CircleProps extends Shape {
  cx?: Length;
  cy?: Length;
  r: Length;
}

export interface EllipseProps extends Shape {
  cx?: Length;
  cy?: Length;
  rx: Length;
  ry: Length;
}

export interface TspanProps extends SVGPresentationAttributes {
  x?: Length;
  y?: Length;
  children?: ReactNode;
}

export interface GProps extends SVGPresentationAttributes {
  style?: Style;
  children?: ReactNode;
}

export interface StopProps {
  offset: Length;
  stopColor: string;
  stopOpacity?: Length;
}

export interface DefsProps {
  children?: ReactNode;
}

export interface ClipPathProps {
  id?: string;
  children?: ReactNode;
}

export interface MarkerProps {
  id: string;
  viewBox?: string;
  refX?: Length;
  refY?: Length;
  markerWidth?: Length;
  markerHeight?: Length;
  orient?: "auto" | "auto-start-reverse" | number;
  markerUnits?: "strokeWidth" | "userSpaceOnUse";
  children?: ReactNode;
}

interface Gradient {
  id: string;
  xlinkHref?: string;
  gradientTransform?: string;
  gradientUnits?: "userSpaceOnUse" | "objectBoundingBox";
  children?: ReactNode;
}

export interface LinearGradientProps extends Gradient {
  x1?: Length;
  x2?: Length;
  y1?: Length;
  y2?: Length;
}

export interface RadialGradientProps extends Gradient {
  cx?: Length;
  cy?: Length;
  r?: Length;
  fx?: Length;
  fy?: Length;
}
