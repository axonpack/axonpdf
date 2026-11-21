import type { TreeNode, TreeText } from "../types/tree.type.js";
import type { Scope } from "./dispatcher.js";
import { serialize } from "./serialize.js";
import type { Session } from "./session.js";
import { walk } from "./walk.js";

/** One `Rows`: hands out its next batch as `{"children":[...],"done":bool}` JSON. */
export interface RowsSource {
  /** An async source makes the render resumable: the engine stops and waits for each batch. */
  async: boolean;
  next(): string | Promise<string>;
}

/**
 * A `ROWS` node: the engine asks for its rows in batches by id as its page cut reaches them.
 * Only the batch being handed over is held here: rows are read and rendered on demand.
 */
export function rows(props: Record<string, any>, session: Session, scope: Scope): TreeNode {
  if (session.inCallback) throw new Error("Rows can't be used inside a render function.");
  const id = session.nextId++;
  const batchSize = Math.max(1, props.batchSize as number);
  let index = 0;
  const batch = (items: unknown[], done: boolean) => {
    const children: (TreeNode | TreeText)[] = [];
    for (const item of items) {
      for (const c of walk(props.renderRow(item, index++), scope)) {
        children.push(serialize(c, session));
      }
    }
    return JSON.stringify({ children, done });
  };

  const source = props.rows;
  if (Symbol.iterator in Object(source)) {
    let iterator: Iterator<unknown> | undefined;
    session.rows.set(id, {
      async: false,
      next() {
        iterator ??= (source as Iterable<unknown>)[Symbol.iterator]();
        const items = [];
        for (let next = iterator.next(); !next.done; next = iterator.next()) {
          items.push(next.value);
          if (items.length === batchSize) return batch(items, false);
        }
        return batch(items, true);
      },
    });
  } else {
    let iterator: AsyncIterator<unknown> | undefined;
    session.rows.set(id, {
      async: true,
      async next() {
        iterator ??= (source as AsyncIterable<unknown>)[Symbol.asyncIterator]();
        const items = [];
        for (let next = await iterator.next(); !next.done; next = await iterator.next()) {
          items.push(next.value);
          if (items.length === batchSize) return batch(items, false);
        }
        return batch(items, true);
      },
    });
  }
  return { type: "ROWS", props: { rows: id, batchSize }, children: [] };
}
