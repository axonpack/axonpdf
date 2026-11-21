import type { Item } from "../types/instance.type.js";
import type { TreeNode } from "../types/tree.type.js";
import { Fallback } from "./dispatcher.js";
import { serialize } from "./serialize.js";
import type { Session } from "./session.js";

/** Children sent with their parent; the rest wait. The engine keeps the same number built. */
const EAGER = 64;
/** Nodes whose children are laid out together, never waiting (as in the engine). */
const WHOLE = new Set(["TEXT", "SVG", "NOTE"]);
/**
 * What the engine never leaves waiting past its first children: anything that repeats on every
 * page, renders itself, counts pages, sits absolutely, marks the outline, or breaks after it.
 */
const NOT_PLAIN = [
  '"render"',
  '"fixed"',
  '"bookmark"',
  "PAGE_NUMBER",
  "PAGE_COUNT",
  "absolute",
  '"breakAfter"',
];
const plain = (json: string) => !NOT_PLAIN.some((word) => json.includes(word));

/**
 * Serializes `node`'s first children now and leaves the rest to be walked when the engine
 * reaches them (`"lazy": id`, asked for in batches like `<Rows>`), so a long list is never held
 * whole. Each batch is built as the engine asks and dropped once handed over. A late child the
 * engine would have built up front (see `NOT_PLAIN`) makes the render fall back.
 */
export function lazyChildren(node: TreeNode, children: Iterator<Item>, session: Session) {
  const take = () => {
    const next = children.next();
    return next.done ? undefined : next.value;
  };
  for (let i = 0; i < EAGER; i++) {
    const item = take();
    if (!item) return;
    node.children.push(serialize(item, session));
  }
  let first = take();
  if (!first) return;
  // A break after any child makes the engine build them all; so do we.
  const breaks = node.children.some((c) => JSON.stringify(c).includes('"breakAfter"'));
  if (WHOLE.has(node.type) || breaks) {
    for (let item: Item | undefined = first; item; item = take()) {
      node.children.push(serialize(item, session));
    }
    return;
  }
  const id = session.nextId++;
  node.props.lazy = id;
  session.rows.set(id, {
    async: false,
    lazy: true,
    next() {
      const batch: string[] = [];
      const outer = session.inBatch;
      session.inBatch = true;
      try {
        for (let item = first ?? take(); item; item = take()) {
          first = undefined;
          const json = JSON.stringify(serialize(item, session));
          if (!plain(json)) throw new Fallback("a late child the engine builds up front");
          batch.push(json);
          if (batch.length === EAGER) return `{"children":[${batch}],"done":false}`;
        }
        session.rows.delete(id);
        return `{"children":[${batch}],"done":true}`;
      } finally {
        session.inBatch = outer;
      }
    },
  });
}

/** Leaves a page's children to be built, all at once, when the engine opens the page. */
export function lazyPage(node: TreeNode, children: Iterable<Item>, session: Session) {
  const id = session.nextId++;
  node.props.lazy = id;
  session.rows.set(id, {
    async: false,
    lazy: true,
    next() {
      session.rows.delete(id);
      const outer = session.inBatch;
      session.inBatch = true;
      try {
        const built = [...children].map((item) => JSON.stringify(serialize(item, session)));
        return `{"children":[${built}],"done":true}`;
      } finally {
        session.inBatch = outer;
      }
    },
  });
}
