import type { Scope } from "../tree/dispatcher.js";

/** A host element as the reconciler builds it: the type, the props as written, the children. */
export interface Instance {
  type: string;
  props: Record<string, any>;
  children: Child[];
}

export interface TextInstance {
  type: "TEXT_INSTANCE";
  value: string;
}

export type Child = Instance | TextInstance;

/**
 * A host element as either renderer hands it over: the reconciler's, or the one-shot walk's,
 * whose children are walked as they are read, under the scope it was found in.
 */
export type Item =
  | { type: string; props: Record<string, any>; children: Iterable<Item>; scope?: Scope }
  | TextInstance;

/** The reconciler's root: holds the `Document` once React commits it. */
export interface Container {
  document: Instance | null;
}
