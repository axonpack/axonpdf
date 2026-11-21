import { expand } from "./expand.js";
import { flatten } from "./flatten.js";
import { resolveMedia } from "./media.js";
import type { Container, SafeStyle, StyleProp } from "./types.js";

/**
 * Resolves a style against a container, as the engine does before layout: lists flattened,
 * matching `@media` blocks applied, shorthands expanded, lengths in points, colors as hex and
 * font weights as numbers.
 */
export default function resolveStyles(
  container: Container,
  style: StyleProp | null | undefined,
): SafeStyle {
  const out: SafeStyle = {};
  for (const [key, value] of Object.entries(resolveMedia(container, flatten(style as never)))) {
    if (value !== undefined) expand(key, value, container, out);
  }
  return out;
}

export { flatten } from "./flatten.js";
export { transformColor } from "./colors.js";
export type * from "./types.js";
