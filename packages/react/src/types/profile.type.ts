import type { Engine, EngineProfile } from "@axonpdf/core";

/** The parts of a render, in the order they run. Each phase's time is its own. */
export interface ProfilePhases {
  /** Your components' own code, wherever they run: while the tree is read, or when lazy
   * content is built during the engine's work. Not `render` props: those are `engine.callbacks`. */
  react: number;
  /** Turning the rendered elements into the engine's input, lazy content's included. */
  tree: number;
  /** Loading images, fonts and emoji, and starting the engine. */
  assets: number;
  /** The engine's call, `engine` splits it. */
  engine: number;
  /** Making `Rows` batches. */
  rows: number;
}

/** Where the time on the CPU went, by function, from V8's sampling profiler (Node only). */
export interface CpuProfile {
  /** Self time by where the code comes from: "your code", a package name, "axonpdf",
   * "built-ins" (V8's own functions, such as `Intl`), "garbage collection", "node". */
  groups: { name: string; ms: number }[];
  /** The functions with the most self time. */
  top: { name: string; group: string; location: string; ms: number }[];
}

export interface ProfileReport {
  /** From the call to the last byte, in milliseconds. */
  totalMs: number;
  phases: ProfilePhases;
  /** The engine phase split by the engine's own phases; `bridge` is the rest: passing data
   * between JavaScript and the engine. */
  engine: Omit<EngineProfile, "pages"> & { bridge: number };
  /** Each page's time to its last byte, in milliseconds. */
  pages: number[];
  /** The size of the PDF in bytes. */
  bytes: number;
  memory: {
    /** The JavaScript heap's largest size seen, at each phase and page; 0 where it can't be read. */
    jsHeapPeak: number;
    /** The engine's WebAssembly memory. */
    wasm: number;
  };
  /** Only in Node. */
  cpu?: CpuProfile;
}

/** Starts and stops a CPU profiler around a render. */
export interface CpuSampler {
  start(): Promise<void>;
  stop(): Promise<CpuProfile>;
}

/** What a render reports to while it is profiled. */
export interface Recorder {
  /** Ends the running phase and starts `phase`. */
  mark(phase: keyof ProfilePhases): void;
  /** The engine, before it renders and once it is done. */
  engine(engine: Engine, done: boolean): void;
  /** Counts the time from now until the returned function is called as `phase`, then goes back
   * to the phase that was running. Fine-grained, so no `performance.measure` for it. */
  enter(phase: keyof ProfilePhases): () => void;
}
