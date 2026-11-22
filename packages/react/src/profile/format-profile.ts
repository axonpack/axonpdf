import type { ProfileReport } from "../types/profile.type.js";

const PHASES = {
  react: "React render (your components)",
  tree: "Reading the tree",
  assets: "Images, fonts, engine start",
  engine: "Engine",
  rows: "Rows batches (your code)",
} as const;

const ENGINE = {
  parse: "parse",
  layout: "layout",
  text: "text",
  paint: "paint",
  write: "write",
  callbacks: "render functions (your code)",
  bridge: "JS and engine bridge",
} as const;

const ms = (n: number) => `${n.toFixed(n < 10 ? 1 : 0)} ms`.padStart(10);
const mb = (bytes: number) => `${(bytes / 2 ** 20).toFixed(1)} MB`;

/** A short text table of a profile: phases, the slowest pages, and in Node the CPU by package
 * and the busiest functions. */
export function formatProfile(report: ProfileReport): string {
  const { totalMs, phases, engine, pages, memory, cpu } = report;
  const share = (n: number) => `${((n / totalMs) * 100).toFixed(1)}%`.padStart(7);
  const row = (label: string, n: number, indent = "") =>
    `${(indent + label).padEnd(34)}${ms(n)}${share(n)}`;
  const lines = [
    `Total ${ms(totalMs).trim()}, ${pages.length} pages, ${(report.bytes / 1024).toFixed(0)} KB PDF`,
    `Memory: JS heap peak ${mb(memory.jsHeapPeak)}, WebAssembly ${mb(memory.wasm)}`,
    "",
  ];
  for (const [key, label] of Object.entries(PHASES)) {
    const n = phases[key as keyof typeof PHASES];
    if (key === "rows" && !n) continue;
    lines.push(row(label, n));
    if (key !== "engine") continue;
    for (const [part, name] of Object.entries(ENGINE)) {
      lines.push(row(name, engine[part as keyof typeof ENGINE], "  "));
    }
  }
  const slowest = pages
    .map((n, i) => ({ n, page: i + 1 }))
    .sort((a, b) => b.n - a.n)
    .slice(0, 3);
  if (slowest.length) {
    lines.push(
      "",
      `Slowest pages: ${slowest.map((p) => `${p.page} (${ms(p.n).trim()})`).join(", ")}`,
    );
  }
  if (cpu) {
    lines.push("", "CPU self time by where the code comes from:");
    cpu.groups.slice(0, 8).forEach((g) => lines.push(row(g.name, g.ms, "  ")));
    lines.push("", "Busiest functions (self time):");
    cpu.top.slice(0, 10).forEach((f) => {
      const where = f.location ? ` ${f.location}` : "";
      lines.push(`  ${ms(f.ms)}  ${f.name} [${f.group}]${where}`);
    });
  }
  return lines.join("\n");
}
