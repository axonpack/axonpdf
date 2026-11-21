import { Fragment, isValidElement, type ReactNode } from "react";

import type { Child } from "../types/instance.type.js";

const MEMO = Symbol.for("react.memo");
const FORWARD_REF = Symbol.for("react.forward_ref");

/**
 * Turns what a `render` function returns into host instances without the reconciler, by calling
 * components directly. Hooks don't work here, as the docs say for `render`.
 */
export function elementToInstances(node: ReactNode): Child[] {
  if (node == null || typeof node === "boolean") return [];
  if (typeof node === "string" || typeof node === "number" || typeof node === "bigint") {
    return [{ type: "TEXT_INSTANCE", value: String(node) }];
  }
  if (typeof node === "object" && Symbol.iterator in node) {
    return [...(node as Iterable<ReactNode>)].flatMap(elementToInstances);
  }
  if (!isValidElement(node)) return [];

  const type = node.type as any;
  const props = { ...type?.defaultProps, ...(node.props as Record<string, any>) };
  if (typeof type === "string") {
    const { children, ...rest } = props;
    return [{ type, props: rest, children: elementToInstances(children) }];
  }
  if (type === Fragment) return elementToInstances(props.children);
  if (typeof type === "function") {
    const output = type.prototype?.isReactComponent ? new type(props).render() : type(props);
    return elementToInstances(output);
  }
  if (type?.$$typeof === MEMO) return elementToInstances({ ...node, type: type.type } as ReactNode);
  if (type?.$$typeof === FORWARD_REF) return elementToInstances(type.render(props, null));
  // Context providers and anything else that only wraps its children.
  return elementToInstances(props.children);
}
