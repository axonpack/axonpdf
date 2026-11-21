import type { RenderArgs } from "@axonpdf/core";

import type { Session } from "../tree/session.js";

/** What the engine calls each `render` function with first, to see if it reads the total. */
const FIRST: RenderArgs = {
  pageNumber: 1,
  totalPages: undefined,
  subPageNumber: undefined,
  subPageTotalPages: undefined,
  isLast: undefined,
};

/** Whether a page count was built, or a function kept from id `from` on reads the total. */
export function readsTotal(session: Session, from = 0): boolean {
  if (session.pageCount) return true;
  // A copy: what a function returns can add functions of its own.
  // oxlint-disable-next-line unicorn/no-useless-spread
  for (const [id, call] of [...session.callbacks]) {
    if (id < from) continue;
    try {
      if (JSON.parse(call(FIRST)).readsTotal) return true;
    } catch {
      // One that can't run without a total needs it.
      return true;
    }
  }
  return false;
}

/**
 * Whether lazy content reads the total. It can hide what reads it, and once a page is written it
 * is too late to count, so every lazy batch is built here once and thrown away, until something
 * reads the total or nothing is left. That also loads the images and emoji the batches use.
 * The session is spent afterwards.
 */
export function hiddenReadsTotal(session: Session): boolean {
  for (;;) {
    const lazy = [...session.rows.values()].find((source) => source.lazy);
    if (!lazy) return false;
    const from = session.nextId;
    lazy.next();
    if (readsTotal(session, from)) return true;
  }
}
