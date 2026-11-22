type Length = string | number;

/** What SVG shapes paint with, as props or in `style`. */
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
