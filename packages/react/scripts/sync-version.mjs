// Writes package.json's version into src/shared.ts, which exports it as `version`. Runs before
// tsc in `build` (so a canary snapshot builds with its own version) and after `changeset version`.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dirname, "..");
const { version } = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const file = join(root, "src/shared.ts");
const code = readFileSync(file, "utf8");
const next = code.replace(/export const version = ".*";/, `export const version = "${version}";`);
if (next === code && !code.includes(`version = "${version}"`)) throw new Error("no version line");
writeFileSync(file, next);
