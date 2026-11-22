import FontStore from "./font-store.js";

export default FontStore;
export { fontWeightNumber, isStandardFont } from "./font-store.js";
export type { FontLoader, FontStoreType } from "./font-store.js";
export type * from "./types.js";

const KEY = Symbol.for("axonpdf.font-store");

/**
 * The store `@axonpdf/react` reads as `Font`, kept under a global key so that every copy of this
 * package in an app (two installs, two bundles) registers into the same one.
 */
const global = globalThis as typeof globalThis & { [KEY]?: FontStore };
global[KEY] ??= new FontStore();
export const sharedFontStore: FontStore = global[KEY];
