// Copies the three react-reconciler builds, each with its scheduler, into build/vendor/ and
// points the built renderer at them. Shipped inside the package, they install no npm packages of
// their own, so no reconciler's peer range on `react` warns in an app on another React version.
// Run after tsc (`bun run build`). React's MIT license travels with each copy.
import { cpSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

const root = join(import.meta.dirname, "..");
const vendor = join(root, "build/vendor");
const require = createRequire(join(root, "package.json"));
const builds = ["react-reconciler-18", "react-reconciler-19-0", "react-reconciler-19-2"];

const copy = (from, to, files) =>
  files.forEach((file) => cpSync(join(from, file), join(to, file), { recursive: true }));
// The reconciler and scheduler code, without the profiling and other side entries.
const main = (dir, name) =>
  readdirSync(join(dir, "cjs"))
    .filter((f) => f.startsWith(`${name}.`) && !f.includes("profiling"))
    .map((f) => `cjs/${f}`);

mkdirSync(vendor, { recursive: true });
writeFileSync(join(vendor, "package.json"), '{ "type": "commonjs" }\n');
for (const name of builds) {
  const from = dirname(require.resolve(`${name}/package.json`));
  const to = join(vendor, name);
  mkdirSync(join(to, "cjs"), { recursive: true });
  copy(from, to, ["index.js", "LICENSE", ...main(from, "react-reconciler")]);
  const scheduler = dirname(
    createRequire(join(from, "package.json")).resolve("scheduler/package.json"),
  );
  mkdirSync(join(to, "scheduler/cjs"), { recursive: true });
  copy(scheduler, join(to, "scheduler"), ["index.js", "LICENSE", ...main(scheduler, "scheduler")]);
  for (const file of main(from, "react-reconciler")) {
    const path = join(to, file);
    const code = readFileSync(path, "utf8").replace(
      /require\((["'])scheduler\1\)/g,
      'require("../scheduler/index.js")',
    );
    writeFileSync(path, code);
  }
}

const renderer = join(root, "build/reconciler/renderer.js");
let code = readFileSync(renderer, "utf8");
for (const name of builds)
  code = code.replace(`from "${name}"`, `from "../vendor/${name}/index.js"`);
if (builds.some((name) => code.includes(`"${name}"`)))
  throw new Error("a reconciler import was not rewritten");
writeFileSync(renderer, code);
