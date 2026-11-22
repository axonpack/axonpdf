// Puts what @axonpdf/react needs inside its own build, so the published package installs none
// of it. Run after tsc (`bun run build`).
//
// - The three react-reconciler builds, each with its scheduler, go to build/vendor/: no
//   reconciler's peer range on `react` then warns in an app on another React version. React's
//   MIT license travels with each copy.
// - The workspace's private packages it uses (@axonpdf/font, @axonpdf/types) go to
//   build/internal/, and every import of them in the build, .d.ts files included, points there.
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, relative } from "node:path";

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

// The private packages, built: their JavaScript and declarations, no source maps.
const internal = ["font", "types"];
for (const name of internal) {
  const from = join(root, `../${name}/build`);
  if (!existsSync(from)) throw new Error(`build packages/${name} first`);
  cpSync(from, join(root, "build/internal", name), {
    recursive: true,
    filter: (path) => !path.endsWith(".map"),
  });
}

/** Every built .js and .d.ts file, the copies included. */
function* files(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory() && entry.name !== "vendor") yield* files(path);
    else if (/\.(js|d\.ts)$/.test(entry.name)) yield path;
  }
}
for (const file of files(join(root, "build"))) {
  let text = readFileSync(file, "utf8");
  const before = text;
  for (const name of internal) {
    let target = relative(dirname(file), join(root, "build/internal", name, "index.js"));
    if (!target.startsWith(".")) target = `./${target}`;
    text = text.replaceAll(`"@axonpdf/${name}"`, `"${target}"`);
  }
  if (text !== before) writeFileSync(file, text);
}
