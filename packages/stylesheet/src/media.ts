import type { Container, Style } from "./types.js";

/** Whether one `@media` condition (`max-width: 500 and orientation: landscape`) holds. */
function matches(query: string, container: Container): boolean {
  const orientation =
    container.orientation ?? (container.width > container.height ? "landscape" : "portrait");
  return query
    .replace(/^@media\s*/, "")
    .split(/\s+and\s+/)
    .every((part) => {
      const [feature, value] = part
        .replace(/[()]/g, "")
        .split(":")
        .map((s) => s.trim());
      const n = Number.parseFloat(value);
      switch (feature) {
        case "min-width":
          return container.width >= n;
        case "max-width":
          return container.width <= n;
        case "min-height":
          return container.height >= n;
        case "max-height":
          return container.height <= n;
        case "orientation":
          return orientation === value;
        default:
          return false;
      }
    });
}

/** Applies the `@media` blocks that match the container, in order, and drops the rest. */
export function resolveMedia(container: Container, style: Style): Style {
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(style)) {
    if (!key.startsWith("@media")) out[key] = value;
    else if (matches(key, container)) Object.assign(out, value);
  }
  return out as Style;
}
