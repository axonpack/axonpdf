import { defineConfig } from "oxlint";
import web from "oxlint-config-universe/web";

export default defineConfig({
  extends: [web],
  ignorePatterns: ["**/build", "**/dist", "packages/core/wasm"],
  rules: {
    // Single-line guard clauses (`if (!x) return;`) are the house style.
    curly: "off",
  },
  overrides: [
    {
      // tsc's noUnusedLocals and noUnusedParameters already fail the build on these.
      files: ["**/*.ts", "**/*.tsx"],
      rules: { "no-unused-vars": "off", "typescript/no-unused-vars": "off" },
    },
  ],
});
