import type { Child, Item, TextInstance } from "../types/instance.type.js";

/** The hosts that hold strings. */
const TEXT_PARENTS = new Set(["TEXT", "LINK", "TSPAN", "NOTE"]);

/**
 * Whether `child` may sit in a `parent` host. A string anywhere but in text (say
 * `<>{name && <Text>{name}</Text>}</>` with an empty `name`) is warned about and dropped.
 */
export function fitsIn(parent: string, child: Child | Item): boolean {
  if (child.type !== "TEXT_INSTANCE" || TEXT_PARENTS.has(parent)) return true;
  console.warn(`Invalid '${(child as TextInstance).value}' string child outside <Text> component`);
  return false;
}
