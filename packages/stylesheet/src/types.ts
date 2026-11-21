export type { Style, StyleProp, Styles, Transform } from "@axonpdf/types";

/** The box styles resolve against: the page, in points. */
export interface Container {
  width: number;
  height: number;
  /** Dots per inch for `px` (default 72). */
  dpi?: number;
  /** Points per `rem` (default 18). */
  remBase?: number;
  orientation?: "landscape" | "portrait";
}

/** A resolved style: shorthands expanded, units in points, colors as hex. */
export type SafeStyle = Record<string, any>;
