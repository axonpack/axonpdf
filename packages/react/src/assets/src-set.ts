/**
 * What an Image or ImageBackground draws. With `srcSet` (`"a.jpg 300w, b.jpg 600w"`), the
 * narrowest candidate at least as wide as the image is shown, else the widest; the width shown
 * is `sizes` (points, or entries like `"(max-width: 400px) 200px, 50vw"` matched against the
 * page), or the page's width. Only `w` descriptors count; with none, `src` or `source`.
 */
export function imageSource(props: Record<string, any>, pageWidth: number): unknown {
  const candidates = String(props.srcSet ?? "")
    .split(",")
    .flatMap((entry) => {
      const [uri, descriptor = ""] = entry.trim().split(/\s+/);
      const width = descriptor.endsWith("w") ? parseFloat(descriptor) : NaN;
      return uri && !Number.isNaN(width) ? [{ uri, width }] : [];
    })
    .sort((a, b) => a.width - b.width);
  if (!candidates.length) return props.src ?? props.source;
  const shown = shownWidth(props.sizes, pageWidth) || pageWidth;
  return (candidates.find((c) => c.width >= shown) ?? candidates.at(-1)!).uri;
}

/** `sizes` on this page: a number, or the length of the first entry whose condition holds. */
function shownWidth(sizes: unknown, pageWidth: number): number | undefined {
  if (typeof sizes === "number") return sizes;
  if (typeof sizes !== "string") return undefined;
  for (const entry of sizes.split(",").map((e) => e.trim())) {
    const conditional = /^\(([^)]+)\)\s+(.+)$/.exec(entry);
    if (!conditional) return length(entry, pageWidth);
    const bound = /^\s*(min|max)-width\s*:\s*(.+?)\s*$/i.exec(conditional[1]);
    const limit = bound && length(bound[2], pageWidth);
    if (limit == null) continue;
    const holds = bound![1].toLowerCase() === "min" ? pageWidth >= limit : pageWidth <= limit;
    if (holds) return length(conditional[2], pageWidth);
  }
  return undefined;
}

/** A length in points: `vw` and `%` are of the page's width, and `px` counts as a point. */
function length(text: string, pageWidth: number): number | undefined {
  const match = /^(-?\d*\.?\d+)\s*(px|pt|in|mm|cm|vw|%)?$/i.exec(text.trim());
  if (!match) return undefined;
  const per: Record<string, number> = {
    in: 72,
    mm: 72 / 25.4,
    cm: 72 / 2.54,
    vw: pageWidth / 100,
    "%": pageWidth / 100,
  };
  return parseFloat(match[1]) * (per[match[2]?.toLowerCase() ?? ""] ?? 1);
}
