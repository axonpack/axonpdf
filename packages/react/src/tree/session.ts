import type { RenderArgs } from "@axonpdf/core";

/** Everything one render collects next to the tree: functions, images and font families. */
export interface Session {
  nextId: number;
  /** Functions the engine calls back by id; each returns the render result as JSON. */
  callbacks: Map<number, (args: RenderArgs) => string>;
  /** Canvas `paint` functions by id; each returns the recorded painter calls as JSON. */
  paints: Map<number, (width: number | undefined, height: number | undefined) => string>;
  /** Hyphenation functions by id: `(word, builtin) => parts`. */
  hyphenators: Map<number, (word: string, builtin: (word: string) => string[]) => string[]>;
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
}

export const createSession = (): Session => ({
  nextId: 1,
  callbacks: new Map(),
  paints: new Map(),
  hyphenators: new Map(),
  emoji: new Set(),
  assets: new Map(),
  assetIds: new Map(),
  pending: [],
  families: new Set(),
  inCallback: false,
});

/** Waits for pending work, including work that pending work adds. */
export async function settle(session: Session) {
  while (session.pending.length) {
    const batch = session.pending.splice(0);
    await Promise.all(batch);
  }
}
