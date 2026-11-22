import { transformColor } from "./colors.js";
import { parseTransform } from "./transform.js";
import type { Container, SafeStyle } from "./types.js";
import { toPoints } from "./units.js";

const SIDES = ["Top", "Right", "Bottom", "Left"] as const;
const CORNERS = ["TopLeft", "TopRight", "BottomRight", "BottomLeft"] as const;
const WEIGHTS: Record<string, number> = {
  thin: 100,
  hairline: 100,
  ultralight: 200,
  extralight: 200,
  light: 300,
  normal: 400,
  medium: 500,
  semibold: 600,
  demibold: 600,
  bold: 700,
  ultrabold: 800,
  extrabold: 800,
  heavy: 900,
  black: 900,
};
const BORDER_STYLES = new Set(["solid", "dashed", "dotted"]);

const words = (value: unknown) => String(value).trim().split(/\s+/);

/** 1 to 4 values, CSS order, as one per side. */
function sides(value: unknown, c: Container) {
  const v = typeof value === "number" ? [value] : words(value).map((w) => toPoints(w, c));
  const [top, right = top, bottom = top, left = right] = v;
  return [top, right, bottom, left];
}

/** A border shorthand (`2 solid red`) as width, style and color, in any order. */
function border(value: unknown, c: Container) {
  const out: { Width?: unknown; Style?: unknown; Color?: unknown } = {};
  for (const word of typeof value === "number" ? [String(value)] : words(value)) {
    if (BORDER_STYLES.has(word)) out.Style = word;
    else if (/^-?\d|^\./.test(word)) out.Width = toPoints(word, c);
    else out.Color = transformColor(word);
  }
  return out;
}

const ORIGIN: Record<string, string> = {
  center: "50%",
  left: "0%",
  right: "100%",
  top: "0%",
  bottom: "100%",
};

/** One style key, expanded into the keys it stands for. */
export function expand(key: string, value: unknown, c: Container, out: SafeStyle) {
  const length = (v: unknown) => toPoints(v, c);
  if (key === "margin" || key === "padding") {
    sides(value, c).forEach((v, i) => (out[`${key}${SIDES[i]}`] = v));
  } else if (/^(margin|padding)(Horizontal|Vertical)$/.test(key)) {
    const [, base, axis] = /^(margin|padding)(Horizontal|Vertical)$/.exec(key)!;
    const pair = axis === "Horizontal" ? ["Left", "Right"] : ["Top", "Bottom"];
    pair.forEach((side) => (out[`${base}${side}`] = length(value)));
  } else if (key === "border" || /^border(Top|Right|Bottom|Left)$/.test(key)) {
    const which = key === "border" ? SIDES : [key.slice(6) as (typeof SIDES)[number]];
    const parts = border(value, c);
    for (const side of which) {
      for (const [part, v] of Object.entries(parts)) out[`border${side}${part}`] = v;
    }
  } else if (key === "borderWidth" || key === "borderStyle" || key === "borderColor") {
    const part = key.slice(6);
    const values = part === "Width" ? sides(value, c) : words(value);
    const [top, right = top, bottom = top, left = right] = values;
    [top, right, bottom, left].forEach((v, i) => {
      out[`border${SIDES[i]}${part}`] = part === "Color" ? transformColor(String(v)) : v;
    });
  } else if (key === "borderRadius") {
    sides(value, c).forEach((v, i) => (out[`border${CORNERS[i]}Radius`] = v));
  } else if (key === "flex" && typeof value === "string" && words(value).length > 1) {
    const [grow, shrink, basis] = words(value);
    out.flexGrow = Number.parseFloat(grow);
    if (shrink !== undefined) out.flexShrink = Number.parseFloat(shrink);
    if (basis !== undefined) out.flexBasis = length(basis);
  } else if (key === "flex") {
    out.flexGrow = typeof value === "string" ? Number.parseFloat(value) : value;
  } else if (key === "flexFlow") {
    for (const word of words(value)) {
      if (word.includes("wrap")) out.flexWrap = word;
      else out.flexDirection = word;
    }
  } else if (key === "gap") {
    const [row, column = row] = words(value).map(length);
    Object.assign(out, { rowGap: row, columnGap: column });
  } else if (key === "objectPosition" || key === "transformOrigin") {
    const [x, y = key === "objectPosition" ? x : "50%"] = words(value).map((w) =>
      key === "transformOrigin" ? (ORIGIN[w] ?? w) : w,
    );
    const axis = (v: string) => {
      if (key === "objectPosition" && v.endsWith("%")) return Number.parseFloat(v) / 100;
      return length(v);
    };
    Object.assign(out, { [`${key}X`]: axis(x), [`${key}Y`]: axis(y) });
  } else if (key === "transform" || key === "gradientTransform") {
    out[key] = parseTransform(value, c);
  } else if (key === "fontWeight") {
    out[key] = typeof value === "string" ? (WEIGHTS[value] ?? Number.parseFloat(value)) : value;
  } else if (/color$/i.test(key) || key === "fill" || key === "stroke") {
    out[key] = typeof value === "string" ? transformColor(value) : value;
  } else {
    out[key] = length(value);
  }
}
