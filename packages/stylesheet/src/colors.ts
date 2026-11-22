const hex = (n: number) =>
  Math.round(Math.min(255, Math.max(0, n)))
    .toString(16)
    .toUpperCase()
    .padStart(2, "0");

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return [f(0) * 255, f(8) * 255, f(4) * 255];
}

/** `rgb()`, `rgba()`, `hsl()` and `hsla()` as hex (`#RRGGBB`, or `#RRGGBBAA` with alpha).
 * Names and hex colors are left as written. */
export function transformColor(value: string): string {
  const match = /^(rgba?|hsla?)\(([^)]*)\)$/i.exec(value.trim());
  if (!match) return value;
  const parts = match[2].split(/[\s,/]+/).filter(Boolean);
  const number = (part: string, scale: number) =>
    part.endsWith("%") ? (Number.parseFloat(part) / 100) * scale : Number.parseFloat(part);
  const rgb = match[1].toLowerCase().startsWith("rgb")
    ? parts.slice(0, 3).map((p) => number(p, 255))
    : hslToRgb(
        Number.parseFloat(parts[0]),
        number(parts[1], 1) / (parts[1].endsWith("%") ? 1 : 100),
        number(parts[2], 1) / (parts[2].endsWith("%") ? 1 : 100),
      );
  const alpha = parts[3] === undefined ? "" : hex(number(parts[3], 1) * 255);
  return `#${rgb.map(hex).join("")}${alpha}`;
}
