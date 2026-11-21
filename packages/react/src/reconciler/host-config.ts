import { createContext } from "react";

import type { Child, Container, Instance, TextInstance } from "../types/instance.type.js";

// React 19's DefaultLane; React 18 reports its own (16) through getCurrentEventPriority.
const DefaultEventPriority = 32;
const noop = () => {};

const propsWithoutChildren = ({ children: _, ...props }: Record<string, any>) => props;

function remove<T>(list: T[], item: T) {
  const index = list.indexOf(item);
  if (index !== -1) list.splice(index, 1);
}

function insert(list: Child[], child: Child, before: Child) {
  remove(list, child);
  const index = list.indexOf(before);
  list.splice(index === -1 ? list.length : index, 0, child);
}

/** The mutation host config shared by every React version. `onChange` runs after each commit. */
export function createHostConfig(onChange: (container: Container) => void) {
  let updatePriority = 0;
  return {
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: false,
    supportsMicrotasks: true,
    scheduleMicrotask: queueMicrotask,
    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    noTimeout: -1,
    warnsIfNotActing: false,

    getRootHostContext: () => ({}),
    getChildHostContext: (context: object) => context,
    getPublicInstance: (instance: Child) => instance,
    prepareForCommit: () => null,
    resetAfterCommit: onChange,
    preparePortalMount: noop,
    clearContainer: noop,
    detachDeletedInstance: noop,
    shouldSetTextContent: () => false,
    finalizeInitialChildren: () => false,
    commitMount: noop,
    resetTextContent: noop,
    hideInstance: noop,
    unhideInstance: noop,
    hideTextInstance: noop,
    unhideTextInstance: noop,
    getInstanceFromNode: () => null,
    beforeActiveInstanceBlur: noop,
    afterActiveInstanceBlur: noop,
    prepareScopeUpdate: noop,
    getInstanceFromScope: () => null,

    createInstance: (type: string, props: Record<string, any>): Instance => ({
      type,
      props: propsWithoutChildren(props),
      children: [],
    }),
    createTextInstance: (value: string): TextInstance => ({ type: "TEXT_INSTANCE", value }),
    appendInitialChild: (parent: Instance, child: Child) => {
      parent.children.push(child);
    },
    appendChild: (parent: Instance, child: Child) => {
      remove(parent.children, child);
      parent.children.push(child);
    },
    appendChildToContainer: (container: Container, child: Instance) => {
      container.document = child;
    },
    insertBefore: (parent: Instance, child: Child, before: Child) =>
      insert(parent.children, child, before),
    insertInContainerBefore: (container: Container, child: Instance) => {
      container.document = child;
    },
    removeChild: (parent: Instance, child: Child) => remove(parent.children, child),
    removeChildFromContainer: (container: Container) => {
      container.document = null;
    },
    commitTextUpdate: (instance: TextInstance, _old: string, value: string) => {
      instance.value = value;
    },

    // React 18 only
    getCurrentEventPriority: () => 16,
    // React 19 only
    getCurrentUpdatePriority: () => updatePriority,
    setCurrentUpdatePriority: (priority: number) => {
      updatePriority = priority;
    },
    resolveUpdatePriority: () => updatePriority || DefaultEventPriority,
    maySuspendCommit: () => false,
    maySuspendCommitOnUpdate: () => false,
    maySuspendCommitInSyncRender: () => false,
    preloadInstance: () => true,
    startSuspendingCommit: noop,
    suspendInstance: noop,
    waitForCommitToBeReady: () => null,
    getSuspendedCommitReason: () => null,
    NotPendingTransition: null,
    HostTransitionContext: createContext(null),
    resetFormInstance: noop,
    requestPostPaintCallback: noop,
    shouldAttemptEagerTransition: () => false,
    trackSchedulerEvent: noop,
    resolveEventType: () => null,
    resolveEventTimeStamp: () => -1.1,
  };
}

/** React 18 diffs in `prepareUpdate` and passes the result to `commitUpdate`. */
export const react18Update = {
  prepareUpdate: () => true,
  commitUpdate(instance: Instance, _payload: unknown, _type: string, _old: object, props: object) {
    instance.props = propsWithoutChildren(props);
  },
};

/** React 19 calls `commitUpdate` with the props directly. */
export const react19Update = {
  commitUpdate(instance: Instance, _type: string, _old: object, props: object) {
    instance.props = propsWithoutChildren(props);
  },
};
