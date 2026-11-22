import { version, type ReactElement } from "react";
import createReconciler18 from "react-reconciler-18";
import createReconciler190 from "react-reconciler-19-0";
import createReconciler192 from "react-reconciler-19-2";

import type { Container } from "../types/instance.type.js";
import { createHostConfig, react18Update, react19Update } from "./host-config.js";

/** What `createRenderer` returns. */
export interface Renderer {
  /** Creates a root; `onError` (default `console.error`) gets the errors React catches. */
  createContainer(container: Container, onError?: (error: unknown) => void): unknown;
  /** Renders `element` into the root synchronously; `parent` is React's parent component. */
  updateContainer(
    element: ReactElement | null,
    root: unknown,
    parent?: unknown,
    callback?: () => void,
  ): void;
}

const [major, minor] = version.split(".").map(Number);
const LegacyRoot = 0;

/** A renderer for the installed React: one reconciler build per React release line. */
export function createRenderer({
  onChange,
}: {
  onChange: (container: Container) => void;
}): Renderer {
  const config = createHostConfig(onChange);
  if (major < 19) {
    const reconciler = createReconciler18({ ...config, ...react18Update });
    return {
      createContainer: (container, onError = console.error) =>
        reconciler.createContainer(container, LegacyRoot, null, false, null, "", onError, null),
      updateContainer: (element, root, parent = null, callback) =>
        reconciler.updateContainer(element, root, parent as null, callback),
    };
  }
  const create = major === 19 && minor < 2 ? createReconciler190 : createReconciler192;
  const reconciler = create({ ...config, ...react19Update });
  // 19.0 takes transitionCallbacks last, 19.2 a default transition indicator.
  const lastArg = create === createReconciler190 ? null : () => {};
  return {
    createContainer: (container, onError = console.error) =>
      reconciler.createContainer(
        container,
        LegacyRoot,
        null,
        false,
        null,
        "",
        onError,
        onError,
        onError,
        lastArg,
      ),
    updateContainer: (element, root, parent = null, callback) => {
      reconciler.updateContainerSync(element, root, parent as null, callback);
      reconciler.flushSyncWork();
    },
  };
}
