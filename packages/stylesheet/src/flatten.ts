import type { Style } from "./types.js";

type Nested = Style | null | undefined | false | readonly Nested[];

/** Merges a style or a nested list of styles, left to right, skipping empty entries. */
export function flatten(style: Nested): Style {
  if (!style) return {};
  if (!Array.isArray(style)) return { ...(style as Style) };
  return (style as readonly Nested[]).reduce<Style>(
    (merged, s) => ({ ...merged, ...flatten(s) }),
    {},
  );
}
