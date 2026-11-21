/** The painter methods a Canvas `paint` function can call, all chainable. */
const METHODS = [
  "dash",
  "clip",
  "save",
  "path",
  "fill",
  "font",
  "text",
  "rect",
  "scale",
  "moveTo",
  "lineTo",
  "stroke",
  "rotate",
  "circle",
  "lineCap",
  "opacity",
  "ellipse",
  "polygon",
  "restore",
  "lineJoin",
  "fontSize",
  "fillColor",
  "lineWidth",
  "translate",
  "miterLimit",
  "strokeColor",
  "fillOpacity",
  "roundedRect",
  "fillAndStroke",
  "strokeOpacity",
  "bezierCurveTo",
  "quadraticCurveTo",
] as const;

type Call = [string, ...unknown[]];

/**
 * Runs `paint` against a painter that records every call in order, and returns the calls as
 * `[method, ...args]`. A gradient is recorded as `["linearGradient", id, ...args]` (or
 * `radialGradient`), its stops as `["stop", id, offset, color, opacity?]`, and passing it as a
 * color sends `{ "gradient": id }`.
 */
export function recordPaint(
  paint: (painter: any, width: number, height: number) => unknown,
  width: number,
  height: number,
): Call[] {
  const calls: Call[] = [];
  let gradients = 0;
  const arg = (value: unknown) =>
    value && typeof value === "object" && "gradient" in value
      ? { gradient: value.gradient }
      : value;
  const painter: Record<string, (...args: unknown[]) => unknown> = {};
  for (const method of METHODS) {
    painter[method] = (...args) => {
      calls.push([method, ...args.map(arg)]);
      return painter;
    };
  }
  for (const kind of ["linearGradient", "radialGradient"]) {
    painter[kind] = (...args) => {
      const gradient = gradients++;
      calls.push([kind, gradient, ...args]);
      const handle = {
        gradient,
        stop: (offset: number, color: string, opacity?: number) => {
          calls.push(
            opacity === undefined
              ? ["stop", gradient, offset, color]
              : ["stop", gradient, offset, color, opacity],
          );
          return handle;
        },
      };
      return handle;
    };
  }
  paint(painter, width, height);
  return calls;
}
