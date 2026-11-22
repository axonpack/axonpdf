import { Fragment, isValidElement, type ReactNode } from "react";

import type { Item } from "../types/instance.type.js";
import { Fallback, callWithHooks, major, readContext, type Scope } from "./dispatcher.js";
import { fitsIn } from "./stray-text.js";

const is = (name: string) => Symbol.for(`react.${name}`);
const [MEMO, FORWARD_REF, CONTEXT, CONSUMER, PROVIDER, LAZY] = [
  "memo",
  "forward_ref",
  "context",
  "consumer",
  "provider",
  "lazy",
].map(is);
// Element types that only wrap their children when nothing suspends.
const WRAPPERS = new Set<unknown>([
  Fragment,
  ...["strict_mode", "profiler", "suspense", "suspense_list", "activity"].map(is),
]);

/**
 * The host elements and text a React node renders to, in order, each yielded as it is reached:
 * components run (with hooks, context, memo and forwardRef) only when the walk gets to them, so
 * a caller can stop and resume anywhere. A host's children are walked when its own `children`
 * are iterated. Throws `Fallback` for what only the reconciler can render.
 */
export function* walk(node: ReactNode, scope: Scope): Generator<Item> {
  if (node == null || typeof node === "boolean") return;
  if (typeof node === "string" || typeof node === "number" || typeof node === "bigint") {
    // React makes no text node for an empty string.
    if (node !== "") yield { type: "TEXT_INSTANCE", value: String(node) };
    return;
  }
  if (typeof node === "object" && Symbol.iterator in node) {
    for (const child of node as Iterable<ReactNode>) yield* walk(child, scope);
    return;
  }
  if (!isValidElement(node)) throw new Fallback("not a React child");
  yield* element(node.type, node.props as Record<string, any>, scope, (node as any).ref);
}

function* element(
  type: any,
  props: Record<string, any>,
  scope: Scope,
  ref?: unknown,
): Generator<Item> {
  if (typeof type === "string") {
    const { children, ...rest } = props;
    yield { type, props: rest, children: within(type, walk(children, scope)), scope };
  } else if (WRAPPERS.has(type)) {
    yield* walk(props.children, scope);
  } else if (typeof type === "function") {
    const output = type.prototype?.isReactComponent
      ? renderClass(type, props, scope)
      : callWithHooks(scope, () => type(withDefaults(type, props)));
    yield* walk(output, scope);
  } else if (type?.$$typeof === MEMO) {
    yield* element(type.type, props, scope, ref);
  } else if (type?.$$typeof === FORWARD_REF) {
    // From React 19 `ref` is a prop; the render function gets it on its own.
    const { ref: propRef, ...rest } = major >= 19 ? props : { ...props, ref };
    yield* walk(
      callWithHooks(scope, () => type.render(rest, propRef ?? null)),
      scope,
    );
  } else if (type?.$$typeof === PROVIDER || (type?.$$typeof === CONTEXT && major >= 19)) {
    const context = type._context ?? type;
    const contexts = new Map(scope.contexts).set(context, props.value);
    yield* walk(props.children, { ...scope, contexts });
  } else if (type?.$$typeof === CONSUMER || type?.$$typeof === CONTEXT) {
    yield* walk(props.children(readContext(scope, type._context ?? type)), scope);
  } else {
    throw new Fallback(type?.$$typeof === LAZY ? "lazy" : "an unknown element type");
  }
}

/** The children a `parent` host keeps. */
function* within(parent: string, children: Iterable<Item>): Generator<Item> {
  for (const child of children) if (fitsIn(parent, child)) yield child;
}

/** Function components lost `defaultProps` in React 19; before, elements already had them. */
function withDefaults(type: any, props: Record<string, any>) {
  if (major >= 19 || !type.defaultProps) return props;
  return { ...type.defaultProps, ...props };
}

/** Mounts a class component as far as its first render: state, derived state, will-mount. */
function renderClass(Type: any, given: Record<string, any>, scope: Scope): ReactNode {
  if (Type.contextTypes || Type.childContextTypes) throw new Fallback("legacy context");
  const props = { ...given };
  for (const key in Type.defaultProps) props[key] ??= Type.defaultProps[key];
  const context = Type.contextType ? readContext(scope, Type.contextType) : {};
  let rendering = false;
  const queue: unknown[] = [];
  const updater = {
    isMounted: () => false,
    enqueueSetState: (_: unknown, partial: unknown) => {
      if (rendering) throw new Fallback("setState during render");
      queue.push(partial);
    },
    enqueueReplaceState: (_: unknown, state: unknown) => updater.enqueueSetState(_, () => state),
    enqueueForceUpdate: () => {},
  };
  const instance = new Type(props, context);
  Object.assign(instance, { props, context, updater });
  instance.state ??= null;
  const apply = () => {
    for (const partial of queue.splice(0)) {
      const next =
        typeof partial === "function" ? partial.call(instance, instance.state, props) : partial;
      if (next != null) instance.state = { ...instance.state, ...next };
    }
  };
  if (Type.getDerivedStateFromProps) {
    const derived = Type.getDerivedStateFromProps(props, instance.state);
    if (derived != null) instance.state = { ...instance.state, ...derived };
  } else if (!instance.getSnapshotBeforeUpdate) {
    (instance.UNSAFE_componentWillMount ?? instance.componentWillMount)?.call(instance);
    apply();
  }
  rendering = true;
  return callWithHooks(scope, () => instance.render());
}
