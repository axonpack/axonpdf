import type { RenderArgs } from "@axonpdf/core";

import type { RowsSource } from "./rows.js";

/** Everything one render collects next to the tree: functions, images and font families. */
export interface Session {
  nextId: number;
  /** Functions the engine calls back by id; each returns the render result as JSON. */
  callbacks: Map<number, (args: RenderArgs) => string>;
  /** Canvas `paint` functions by id; each returns the recorded painter calls as JSON. */
  paints: Map<number, (width: number | undefined, height: number | undefined) => string>;
  /** Hyphenation functions by id: `(word, builtin) => parts`. */
  hyphenators: Map<number, (word: string, builtin: (word: string) => string[]) => string[]>;
  /** Each `Rows` by id: renders and returns its next batch as JSON. */
  rows: Map<number, RowsSource>;
  /** Every emoji the text holds, for the registered emoji source. */
  emoji: Set<string>;
  /** Image bytes by asset id, for the engine's `add_image`. */
  assets: Map<number, Uint8Array>;
  /** Asset ids by source, so one image used twice is sent once. */
  assetIds: Map<unknown, number>;
  /** Work that must finish before the engine runs (image loads, async rows). */
  pending: Promise<void>[];
  families: Set<string>;
  /** True while the engine is calling back: nothing async can be started then. */
  inCallback: boolean;
  /** True while building lazy children for the engine: an image not loaded yet restarts. */
  inBatch: boolean;
  /** True while building what a `render` function returned: no lazy children there. */
  inRender: boolean;
  /** Each page's content waits until the engine opens the page. */
  lazyPages: boolean;
  /** The props of the page whose content is being built, for `srcSet`. */
  pageProps: object;
  /** A page's width in points from its props; set before the tree is built. */
  measurePage?: (props: object) => number;
  /** A `PageCount` was built. */
  pageCount: boolean;
  /** Kept from an earlier try of the same render (see `render-document.ts`). */
  carried: Carried;
  /** Lazy children met an image or emoji not loaded yet: this try's pages are dropped. */
  restart: boolean;
  /** The asset ids this try gave the carried images, by their key, and how many are used. */
  carriedIds: Map<unknown, number[]>;
  carriedUsed: Map<unknown, number>;
}

/** What one try of a render loads for the next: images by source, emoji images by emoji. */
export interface Carried {
  /** Every image source met, in the order met, with its bytes once loaded. */
  images: { key: unknown; bytes?: Uint8Array; held: boolean }[];
  emoji: Map<string, Uint8Array | null>;
  /** Whether a document with lazy content has its pages counted first, once decided. */
  count?: boolean;
}

/** A try that met images or emoji not loaded yet, to be made again once they are. */
export class Restart extends Error {}

export function createSession(carried: Carried = { images: [], emoji: new Map() }): Session {
  const session: Session = {
    nextId: 1,
    callbacks: new Map(),
    paints: new Map(),
    hyphenators: new Map(),
    rows: new Map(),
    emoji: new Set(),
    assets: new Map(),
    assetIds: new Map(),
    pending: [],
    families: new Set(),
    inCallback: false,
    inBatch: false,
    inRender: false,
    lazyPages: false,
    pageProps: {},
    pageCount: false,
    carried,
    restart: false,
    carriedIds: new Map(),
    carriedUsed: new Map(),
  };
  // Images an earlier try met in lazy children go to the engine with the rest, up front.
  // In the order the engine gets them when nothing waits: bytes held first, then loaded ones.
  const order = [...carried.images.filter((i) => i.held), ...carried.images.filter((i) => !i.held)];
  for (const { key, bytes } of order) {
    if (!bytes) continue;
    const id = session.nextId++;
    session.carriedIds.set(key, [...(session.carriedIds.get(key) ?? []), id]);
    session.assets.set(id, bytes);
  }
  return session;
}

/** Waits for pending work, including work that pending work adds. */
export async function settle(session: Session) {
  while (session.pending.length) {
    const batch = session.pending.splice(0);
    await Promise.all(batch);
  }
}
