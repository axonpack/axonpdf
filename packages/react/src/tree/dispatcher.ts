import * as React from "react";

/** Thrown when the one-shot renderer meets something only the reconciler handles. */
export class Fallback extends Error {}

/** What a component sees from above: context values, and a counter for `useId`. */
export interface Scope {
  contexts: ReadonlyMap<object, unknown>;
  ids: { next: number };
}

export const rootScope = (): Scope => ({ contexts: new Map(), ids: { next: 0 } });

export const major = Number(React.version.split(".")[0]);

// React reads the current hooks dispatcher from its shared internals; servers set it the same way.
const internals: any =
  (React as any).__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE ??
  (React as any).__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED;

function setDispatcher(dispatcher: unknown): unknown {
  if (major >= 19) {
    const previous = internals.H;
    internals.H = dispatcher;
    return previous;
  }
  const previous = internals.ReactCurrentDispatcher.current;
  internals.ReactCurrentDispatcher.current = dispatcher;
  return previous;
}

/** A context's value in `scope`, or its default. */
export function readContext(scope: Scope, context: any): unknown {
  // A secondary renderer (ours) keeps its values in `_currentValue2`; both start as the default.
  return scope.contexts.has(context) ? scope.contexts.get(context) : context._currentValue2;
}

const CONTEXT = Symbol.for("react.context");
const noop = () => {};

/** The hooks for one render pass with no updates: initial state, memos run once, no effects. */
function dispatcher(scope: Scope, rendering: { done: boolean }) {
  const update = () => {
    // A state update while rendering means rendering again: only the reconciler does that.
    if (!rendering.done) throw new Fallback("a state update during render");
  };
  const initial = (value: unknown) => (typeof value === "function" ? value() : value);
  return {
    readContext: (context: unknown) => readContext(scope, context),
    useContext: (context: unknown) => readContext(scope, context),
    use(usable: any) {
      if (usable?.$$typeof === CONTEXT) return readContext(scope, usable);
      if (usable?.status === "fulfilled") return usable.value;
      throw new Fallback("use() of a pending value");
    },
    useState: (value: unknown) => [initial(value), update],
    useReducer: (_: unknown, arg: unknown, init?: (arg: unknown) => unknown) => [
      init ? init(arg) : arg,
      update,
    ],
    useRef: (current: unknown) => ({ current }),
    useMemo: (create: () => unknown) => create(),
    useCallback: (callback: unknown) => callback,
    useEffect: noop,
    useLayoutEffect: noop,
    useInsertionEffect: noop,
    useImperativeHandle: noop,
    useDebugValue: noop,
    useEffectEvent: (callback: unknown) => callback,
    useId: () => `«r${(scope.ids.next++).toString(32)}»`,
    useSyncExternalStore: (_: unknown, getSnapshot: () => unknown) => getSnapshot(),
    useTransition: () => [false, (callback: () => void) => callback()],
    useDeferredValue: (value: unknown) => value,
    useOptimistic: (state: unknown) => [state, update],
    useActionState: (_: unknown, state: unknown) => [state, update, false],
    useFormState: (_: unknown, state: unknown) => [state, update, false],
    useHostTransitionStatus: () => ({ pending: false, data: null, method: null, action: null }),
    useCacheRefresh: () => noop,
    // What compiled components use to memoize: every slot starts empty.
    useMemoCache: (size: number) =>
      Array.from({ length: size }, () => Symbol.for("react.memo_cache_sentinel")),
  };
}

/** Calls a component (or a class's `render`) with hooks answered from `scope`. */
export function callWithHooks<T>(scope: Scope, call: () => T): T {
  const rendering = { done: false };
  const previous = setDispatcher(dispatcher(scope, rendering));
  try {
    return call();
  } finally {
    rendering.done = true;
    setDispatcher(previous);
  }
}
