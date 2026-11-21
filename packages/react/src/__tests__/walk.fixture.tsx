import type { ReactElement } from "react";

import { pdf } from "../pdf.js";
import { rootScope } from "../tree/dispatcher.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession } from "../tree/session.js";
import { walk } from "../tree/walk.js";

const json = (tree: object) =>
  JSON.stringify(tree, (key, value) => (key === "creationDate" ? undefined : value));

/** The tree the one-shot walk builds for `element`. */
export function walked(element: ReactElement<any>) {
  const [root] = walk(element, rootScope());
  return json(serializeDocument(root as any, createSession()));
}

/** The tree the reconciler builds for `element`. */
export function reconciled(element: ReactElement<any>) {
  return json(serializeDocument(pdf(element).container.document!, createSession()));
}

/** Every text in a tree, in order. */
export const texts = (tree: string) =>
  [...tree.matchAll(/"value":"([^"]*)"/g)].map((match) => match[1]);
