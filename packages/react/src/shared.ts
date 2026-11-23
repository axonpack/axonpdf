// What the Node and browser entries both export.
export * from "./components.js";
export { Font } from "./font/font-store.js";
export type { FontDescriptor, FontSource } from "./font/font-store.js";
export { pdf } from "./pdf.js";
export type { PDFInstance } from "./pdf.js";
export { createRenderer } from "./reconciler/renderer.js";
export type { Renderer as PDFRenderer } from "./reconciler/renderer.js";
export { formatProfile } from "./profile/format-profile.js";
export type { CpuProfile, ProfilePhases, ProfileReport } from "./types/profile.type.js";

export const version = "0.0.1";

/** Thrown in Node for a browser-only API, and in the browser for a Node-only one. */
export function environmentError(name: string, environment: "web" | "Node") {
  return () => {
    const where = environment === "web" ? "on Node" : "in a browser";
    throw new Error(
      `${name} is a ${environment} specific API. You're either using this ${environment === "web" ? "component" : "method"} ${where}, or your bundler is not loading axonpdf from the appropriate web build.`,
    );
  };
}
