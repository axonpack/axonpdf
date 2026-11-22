import { Session } from "node:inspector";
import { dirname, relative } from "node:path";
import { fileURLToPath } from "node:url";

import type { CpuProfile, CpuSampler } from "../types/profile.type.js";

interface Profile {
  nodes: { id: number; callFrame: { functionName: string; url: string; lineNumber: number } }[];
  samples: number[];
  timeDeltas: number[];
}

/** This package's folder and the engine's: code from either is axonpdf's. */
const ours = () => [
  fileURLToPath(new URL("../..", import.meta.url)),
  dirname(dirname(fileURLToPath(import.meta.resolve("@axonpdf/core")))),
];

/** Where a function comes from, by its script's URL. */
function groupOf(functionName: string, url: string, roots: string[]): string | undefined {
  if (functionName === "(garbage collector)") return "garbage collection";
  if (functionName.startsWith("(")) return undefined; // idle, program, root
  if (url.startsWith("wasm://")) return "axonpdf";
  if (!url) return "built-ins";
  if (url.startsWith("node:")) return "node";
  const path = url.startsWith("file:") ? fileURLToPath(url) : url;
  if (roots.some((root) => path.startsWith(root)) || path.includes("/@axonpdf/")) return "axonpdf";
  const at = path.lastIndexOf("/node_modules/");
  const [scope, name] = path.slice(at + 14).split("/");
  // Bundlers' caches (`node_modules/.cache`, `.vite`) hold the app's own code.
  if (at < 0 || scope.startsWith(".")) return "your code";
  return scope.startsWith("@") ? `${scope}/${name}` : scope;
}

/** Self time by group and by function, from V8's samples. */
function summarize(profile: Profile): CpuProfile {
  const byNode = new Map<number, number>();
  profile.samples.forEach((id, i) => {
    byNode.set(id, (byNode.get(id) ?? 0) + (profile.timeDeltas[i] ?? 0) / 1000);
  });
  const roots = ours();
  const groups = new Map<string, number>();
  const functions = new Map<string, CpuProfile["top"][number]>();
  for (const { id, callFrame } of profile.nodes) {
    const ms = byNode.get(id);
    const group = ms && groupOf(callFrame.functionName, callFrame.url, roots);
    if (!ms || !group) continue;
    groups.set(group, (groups.get(group) ?? 0) + ms);
    const file = callFrame.url.startsWith("file:") ? fileURLToPath(callFrame.url) : callFrame.url;
    const location = file && `${relative(process.cwd(), file)}:${callFrame.lineNumber + 1}`;
    const name = callFrame.functionName || "(anonymous)";
    const key = `${name} ${location}`;
    const entry = functions.get(key) ?? { name, group, location, ms: 0 };
    entry.ms += ms;
    functions.set(key, entry);
  }
  return {
    groups: [...groups].map(([name, ms]) => ({ name, ms })).sort((a, b) => b.ms - a.ms),
    top: [...functions.values()].sort((a, b) => b.ms - a.ms).slice(0, 15),
  };
}

/** Samples the CPU with V8's profiler through `node:inspector`, every 250 µs. */
export function cpuSampler(): CpuSampler {
  const session = new Session();
  const post = <T>(method: string, params?: object) =>
    new Promise<T>((resolve, reject) =>
      session.post(method, params, (error, result) =>
        error ? reject(error) : resolve(result as T),
      ),
    );
  return {
    async start() {
      session.connect();
      await post("Profiler.enable");
      await post("Profiler.setSamplingInterval", { interval: 250 });
      await post("Profiler.start");
    },
    async stop() {
      const { profile } = await post<{ profile: Profile }>("Profiler.stop");
      session.disconnect();
      return summarize(profile);
    },
  };
}
