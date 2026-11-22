import type { Container, Transform } from "./types.js";
import { toPoints } from "./units.js";

const angle = (value: string) => {
  const n = Number.parseFloat(value);
  return value.trim().endsWith("rad") ? (n * 180) / Math.PI : n;
};

/** One CSS transform function as `{ operation, value }`, the single-axis forms made whole. */
function operation(name: string, args: string[], container: Container): Transform {
  const length = (i: number) => toPoints(args[i] ?? "0", container) as number;
  const number = (i: number) => Number.parseFloat(args[i]);
  switch (name) {
    case "translate":
      return { operation: "translate", value: [length(0), args[1] ? length(1) : 0] };
    case "translateX":
      return { operation: "translate", value: [length(0), 0] };
    case "translateY":
      return { operation: "translate", value: [0, length(0)] };
    case "scale":
      return { operation: "scale", value: [number(0), args[1] ? number(1) : number(0)] };
    case "scaleX":
      return { operation: "scale", value: [number(0), 1] };
    case "scaleY":
      return { operation: "scale", value: [1, number(0)] };
    case "rotate":
      return { operation: "rotate", value: [angle(args[0])] } as unknown as Transform;
    case "skew":
      return { operation: "skew", value: [angle(args[0]), args[1] ? angle(args[1]) : 0] };
    case "skewX":
      return { operation: "skew", value: [angle(args[0]), 0] };
    case "skewY":
      return { operation: "skew", value: [0, angle(args[0])] };
    case "matrix":
      return { operation: "matrix", value: args.map(Number.parseFloat) } as Transform;
    default:
      throw new Error(`Unknown transform: ${name}`);
  }
}

/** A transform string (`translate(10, 20) rotate(45)`) as a list; a list stays as it is. */
export function parseTransform(value: unknown, container: Container): unknown {
  if (typeof value !== "string") return value;
  return [...value.matchAll(/(\w+)\(([^)]*)\)/g)].map(([, name, args]) =>
    operation(name, args.split(/[\s,]+/).filter(Boolean), container),
  );
}
