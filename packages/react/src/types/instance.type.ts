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

/** The reconciler's root: holds the `Document` once React commits it. */
export interface Container {
  document: Instance | null;
}
