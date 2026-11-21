import type { EngineProfile } from "@axonpdf/core";
import type { ReactElement } from "react";

import { mount } from "../pdf.js";
import type { CpuSampler, ProfilePhases, ProfileReport, Recorder } from "../types/profile.type.js";
import type { DocumentProps } from "../types/props.type.js";

/** The JavaScript heap in use, where the platform tells: Node, or Chromium's `performance.memory`. */
function jsHeap(): number {
  const g = globalThis as {
    process?: { memoryUsage?: () => { heapUsed: number } };
    performance: { memory?: { usedJSHeapSize: number } };
  };
  return g.process?.memoryUsage?.().heapUsed ?? g.performance.memory?.usedJSHeapSize ?? 0;
}

/**
 * Renders `element` to nothing while timing each phase, and marks the phases with
 * `performance.measure` so a browser's Performance panel shows them. `sampler` adds CPU samples.
 */
export async function runProfile(
  element: ReactElement<DocumentProps>,
  sampler?: CpuSampler,
): Promise<ProfileReport> {
  const phases: ProfilePhases = { react: 0, tree: 0, assets: 0, engine: 0, rows: 0 };
  let current: keyof ProfilePhases | undefined;
  let last = 0;
  let heapPeak = 0;
  let measured: EngineProfile | undefined;
  let wasm = 0;
  const heap = () => {
    heapPeak = Math.max(heapPeak, jsHeap());
  };
  const close = () => {
    const now = performance.now();
    if (current) {
      phases[current] += now - last;
      performance.measure(`axonpdf ${current}`, { start: last, end: now });
    }
    last = now;
    heap();
  };
  const recorder: Recorder = {
    mark(phase) {
      close();
      current = phase;
    },
    engine(engine, done) {
      if (!done) return engine.profile(true);
      measured = engine.profileReport();
      wasm = engine.memoryBytes();
      engine.profile(false);
    },
  };

  await sampler?.start();
  let bytes = 0;
  let cpu;
  const start = performance.now();
  try {
    recorder.mark("react");
    const { render } = mount(element);
    await render(
      (chunk) => {
        bytes += chunk.length;
        heap();
      },
      undefined,
      true,
      recorder,
    );
    close();
  } finally {
    current = undefined;
    cpu = await sampler?.stop();
  }
  const { pages = [], ...engine } = measured ?? ({} as Partial<EngineProfile>);
  const inside = Object.values(engine).reduce((sum, ms) => sum + ms, 0);
  return {
    totalMs: last - start,
    phases,
    engine: {
      parse: 0,
      layout: 0,
      text: 0,
      paint: 0,
      write: 0,
      callbacks: 0,
      ...engine,
      bridge: Math.max(0, phases.engine - inside),
    },
    pages,
    bytes,
    memory: { jsHeapPeak: heapPeak, wasm },
    cpu,
  };
}
