import { sharedFontStore } from "@axonpdf/font";

import { fetchBytes } from "../assets/fetch-bytes.js";

export { fontWeightNumber, isStandardFont } from "@axonpdf/font";
export type { FontDescriptor, FontSource } from "@axonpdf/font";

// The one font store, shared by every copy of this package in an app. Its fonts load through
// the platform, so Node reads file paths from disk.
sharedFontStore.loader = (src, options) => fetchBytes(src, options);

export const Font = sharedFontStore;
