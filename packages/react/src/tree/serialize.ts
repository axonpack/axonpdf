import type { RenderArgs } from "@axonpdf/core";
import { isValidElement, type ReactNode } from "react";

import { imageBytes } from "../assets/image-bytes.js";
import { Font } from "../font/font-store.js";
import type { Child, Instance, TextInstance } from "../types/instance.type.js";
import type { TreeDocument, TreeNode, TreeText } from "../types/tree.type.js";
import { recordPaint } from "./canvas-painter.js";
import { elementToInstances } from "./element-to-instances.js";
import { rows } from "./rows.js";
import type { Session } from "./session.js";

type Mark = () => void;

const EMOJI = /\p{RGI_Emoji}/gv;

/** The tree JSON for a committed `Document` (shape in `types/tree.type.ts`). */
export function serializeDocument(document: Instance, session: Session): TreeDocument {
  const hyphenationCallback = Font.getHyphenationCallback() ?? undefined;
  const props: Record<string, unknown> = {
    creationDate: new Date(),
    hyphenationCallback,
    ...document.props,
  };
  if (props.userPassword || props.ownerPassword) {
    // The engine has no random source: these seed the encryption keys, salts and IVs.
    props.encryptionSeed = Array.from(crypto.getRandomValues(new Uint8Array(32)));
  }
  return { v: 1, ...(serialize({ ...document, props }, session) as TreeNode) } as TreeDocument;
}

export function serialize(child: Child, session: Session): TreeNode | TreeText {
  if (child.type === "TEXT_INSTANCE") {
    const { value } = child as TextInstance;
    for (const [emoji] of value.matchAll(EMOJI)) session.emoji.add(emoji);
    return { type: "TEXT_INSTANCE", value };
  }
  const { type, props: all, children } = child as Instance;
  if (type === "ROWS") return rows(all, session);

  const node: TreeNode = { type, props: {}, children: [] };
  for (const [key, value] of Object.entries(all)) {
    if (value === undefined) continue;
    if (key === "style") {
      node.style = value;
      collectFamilies(value, session.families);
    } else if (key === "render" && typeof value === "function") {
      node.render = callback(session, value, renderArgs);
    } else if (key === "paint" && type === "CANVAS" && typeof value === "function") {
      const id = session.nextId++;
      session.paints.set(id, (width, height) => JSON.stringify(recordPaint(value, width, height)));
      node.paint = id;
    } else if (key === "hyphenationCallback" && typeof value === "function") {
      const id = session.nextId++;
      session.hyphenators.set(id, value);
      node.props[key] = id;
    } else if ((key === "header" || key === "footer") && type === "PAGE") {
      node.props[key] =
        typeof value === "function"
          ? { render: callback(session, value, bandArgs) }
          : band(value, session);
    } else if ((key === "src" || key === "source") && type.startsWith("IMAGE")) {
      node.props.src = asset(value, all.cache !== false, session);
    } else if (typeof value !== "function") {
      node.props[key] = value;
    }
  }
  node.children = children.map((c) => serialize(c, session));
  return node;
}

const renderArgs = (args: RenderArgs, mark: Mark) => ({
  pageNumber: args.pageNumber,
  subPageNumber: args.subPageNumber,
  get totalPages() {
    mark();
    return args.totalPages;
  },
  get subPageTotalPages() {
    mark();
    return args.subPageTotalPages;
  },
});

const bandArgs = (args: RenderArgs, mark: Mark) => ({
  pageNumber: args.pageNumber,
  isFirst: args.pageNumber === 1,
  isOdd: args.pageNumber % 2 === 1,
  get totalPages() {
    mark();
    return args.totalPages;
  },
  get isLast() {
    mark();
    return args.pageNumber === args.totalPages;
  },
});

/** Keeps `fn` on this side under an id; the engine calls it back while it paginates. */
function callback(
  session: Session,
  fn: (args: any) => ReactNode,
  toArgs: (args: RenderArgs, mark: Mark) => object,
): number {
  const id = session.nextId++;
  session.callbacks.set(id, (args) => {
    let readsTotal = false;
    session.inCallback = true;
    try {
      const output = fn(toArgs(args, () => (readsTotal = true)));
      const children = elementToInstances(output).map((c) => serialize(c, session));
      return JSON.stringify({ children, readsTotal });
    } finally {
      session.inCallback = false;
    }
  });
  return id;
}

function band(element: unknown, session: Session) {
  if (!isValidElement(element)) return null;
  const [first] = elementToInstances(element);
  return first ? serialize(first, session) : null;
}

function asset(source: unknown, useCache: boolean, session: Session) {
  let id = session.assetIds.get(source);
  if (id === undefined) {
    if (session.inCallback) {
      // ponytail: the engine can't take new images mid-render; preload them up front if needed.
      throw new Error("An Image inside a render function must also be used outside it.");
    }
    const assetId = session.nextId++;
    id = assetId;
    session.assetIds.set(source, assetId);
    session.pending.push(
      imageBytes(source, useCache).then((b) => {
        session.assets.set(assetId, b);
      }),
    );
  }
  return { asset: id };
}

function collectFamilies(style: unknown, out: Set<string>) {
  if (!style || typeof style !== "object") return;
  if (Array.isArray(style)) return style.forEach((s) => collectFamilies(s, out));
  for (const [key, value] of Object.entries(style)) {
    if (key === "fontFamily") [value].flat().forEach((f) => typeof f === "string" && out.add(f));
    else if (key.startsWith("@media")) collectFamilies(value, out);
  }
}
