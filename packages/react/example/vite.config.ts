import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  // The workspace link isn't in node_modules, so Vite would serve the package's vendored CommonJS
  // reconciler as ESM. Pre-bundling it, as an installed copy is, converts it.
  optimizeDeps: { include: ["@axonpdf/react"] },
});
