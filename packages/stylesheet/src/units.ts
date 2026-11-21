import type { Container } from "./types.js";

const NUMBER = /^(-?\d*\.?\d+(?:e[+-]?\d+)?)(pt|in|mm|cm|vh|vw|rem|px)?$/i;

/** A length in points: `2in` is 144, `50vw` half the container's width. Percentages, `auto`
 * and anything else that isn't a length stay as written. */
export function toPoints(value: unknown, container: Container): unknown {
  if (typeof value !== "string") return value;
  const match = NUMBER.exec(value.trim());
  if (!match) return value;
  const n = Number.parseFloat(match[1]);
  switch (match[2]?.toLowerCase()) {
    case "in":
      return n * 72;
    case "mm":
      return (n * 72) / 25.4;
    case "cm":
      return (n * 72) / 2.54;
    case "vh":
      return (n * container.height) / 100;
    case "vw":
      return (n * container.width) / 100;
    case "rem":
      return n * (container.remBase ?? 18);
    case "px":
      return (n * 72) / (container.dpi ?? 72);
    default:
      return n;
  }
}
