import type { RenderArgs } from "@axonpdf/core";
import { createElement, isValidElement, type ReactNode } from "react";

import { asset } from "../assets/image-asset.js";
import { Font } from "../font/font-store.js";
import type { Item, TextInstance } from "../types/instance.type.js";
import type { TreeDocument, TreeNode, TreeText } from "../types/tree.type.js";
import { recordPaint } from "./canvas-painter.js";
import { rootScope } from "./dispatcher.js";
import { rows } from "./rows.js";
import { lazyChildren, lazyPage } from "./lazy.js";
import type { Session } from "./session.js";
import { walk } from "./walk.js";

type Mark = () => void;
type Host = Exclude<Item, TextInstance>;

const EMOJI = /\p{RGI_Emoji}/gv;

/** The tree JSON for a committed `Document` (shape in `types/tree.type.ts`). */
export function serializeDocument(document: Host, session: Session): TreeDocument {
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
  // The pages, walked only as far as each <Page>: with more than one, each page's content is
  // built when the engine opens that page.
  const children = [...document.children];
  session.lazyPages = !!document.scope && children.filter((c) => c.type === "PAGE").length > 1;
  const root = serialize({ ...document, props, children } as Item, session);
  return { v: 1, ...(root as TreeNode) } as TreeDocument;
}

export function serialize(child: Item, session: Session): TreeNode | TreeText {
  if (child.type === "TEXT_INSTANCE") {
    const { value } = child as TextInstance;
    for (const [emoji] of value.matchAll(EMOJI)) {
      session.emoji.add(emoji);
      // An emoji first seen in lazy children: fetch its image, then render again.
      if (session.inBatch && Font.getEmojiSource() && !session.carried.emoji.has(emoji)) {
        session.restart = true;
      }
    }
    return { type: "TEXT_INSTANCE", value };
  }
  const { type, props: all, children, scope } = child as Host;
  if (type === "ROWS") return rows(all, session, scope ?? rootScope());

  const node: TreeNode = { type, props: {}, children: [] };
  for (const [key, value] of Object.entries(all)) {
    if (value === undefined) continue;
    if (key === "style") {
      node.style = value;
      collectFamilies(value, session.families);
    } else if (key === "render" && typeof value === "function") {
      node.render = callback(session, value, renderArgs);
    } else if (key === "layout" && type === "PAGE" && typeof value === "function") {
      node.layout = callback(session, value, layoutArgs);
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
  if (scope && !session.inRender && type === "PAGE" && session.lazyPages) {
    lazyPage(node, children, session);
  } else if (scope && !session.inRender) {
    lazyChildren(node, children[Symbol.iterator](), session);
  } else {
    for (const c of children) node.children.push(serialize(c, session));
  }
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

/** A page `layout` template gets the render args plus `children`, where the page content goes. */
const layoutArgs = (args: RenderArgs, mark: Mark) =>
  Object.assign(renderArgs(args, mark), { children: createElement("SLOT") });

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
    return args.isLast ?? args.pageNumber === args.totalPages;
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
    session.inRender = true;
    try {
      const output = fn(toArgs(args, () => (readsTotal = true)));
      const children = [...walk(output, rootScope())].map((c) => serialize(c, session));
      return JSON.stringify({ children, readsTotal });
    } finally {
      session.inCallback = false;
      session.inRender = false;
    }
  });
  return id;
}

function band(element: unknown, session: Session) {
  if (!isValidElement(element)) return null;
  const [first] = walk(element, rootScope());
  return first ? serialize(first, session) : null;
}

function collectFamilies(style: unknown, out: Set<string>) {
  if (!style || typeof style !== "object") return;
  if (Array.isArray(style)) return style.forEach((s) => collectFamilies(s, out));
  for (const [key, value] of Object.entries(style)) {
    if (key === "fontFamily") [value].flat().forEach((f) => typeof f === "string" && out.add(f));
    else if (key.startsWith("@media")) collectFamilies(value, out);
  }
}
