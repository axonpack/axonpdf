import type { TreeNode, TreeText } from "../types/tree.type.js";
import { elementToInstances } from "./element-to-instances.js";
import { serialize } from "./serialize.js";
import type { Session } from "./session.js";

/** A `ROWS` node: the engine pulls its rows in batches by id, as its page cut reaches them. */
export function rows(props: Record<string, any>, session: Session): TreeNode {
  if (session.inCallback) throw new Error("Rows can't be used inside a render function.");
  const id = session.nextId++;
  const batchSize: number = props.batchSize;
  let source: Iterable<unknown> = props.rows;
  if (!(Symbol.iterator in Object(source))) {
    // ponytail: async rows are read in full before the engine starts (in one pass, so a
    // generator works); a resumable engine will pull them as it goes.
    const buffered: unknown[] = [];
    source = buffered;
    session.pending.push(
      (async () => {
        for await (const row of props.rows as AsyncIterable<unknown>) buffered.push(row);
      })(),
    );
  }
  let iterator: Iterator<unknown> | undefined;
  let index = 0;
  // Each row is rendered when its batch is pulled and is not kept here.
  session.rows.set(id, () => {
    iterator ??= source[Symbol.iterator]();
    const children: (TreeNode | TreeText)[] = [];
    const outer = session.inCallback;
    session.inCallback = true;
    try {
      for (let n = 0; n < Math.max(1, batchSize); n++) {
        const next = iterator.next();
        if (next.done) return JSON.stringify({ children, done: true });
        for (const c of elementToInstances(props.renderRow(next.value, index++))) {
          children.push(serialize(c, session));
        }
      }
      return JSON.stringify({ children, done: false });
    } finally {
      session.inCallback = outer;
    }
  });
  return { type: "ROWS", props: { rows: id, batchSize }, children: [] };
}
