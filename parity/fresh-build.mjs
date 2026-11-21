// Our packages load from their build/ folders, so rebuild any whose src/ changed since. The WASM
// build is slow and left to `bun run build:wasm`; a stale one only gets a warning. Returns true
// when the WASM is older than the crates.
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, statSync } from 'node:fs';
import { resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const times = (dir) => (existsSync(dir) ? readdirSync(dir, { recursive: true }).map((f) => statSync(resolve(dir, f)).mtimeMs) : []);
const stale = (src, out) => !times(out).length || Math.max(...times(src)) > Math.min(...times(out));

export function freshBuild() {
  for (const pkg of ['core', 'react']) {
    const dir = resolve(ROOT, 'packages', pkg);
    if (stale(resolve(dir, 'src'), resolve(dir, 'build'))) execFileSync('bun', ['run', 'build'], { cwd: dir, stdio: ['ignore', 'ignore', 'inherit'] });
  }
  const wasmStale = Math.max(...['engine', 'ffi'].flatMap((c) => times(resolve(ROOT, 'crates', c, 'src')))) > Math.max(...times(resolve(ROOT, 'packages/core/wasm')));
  if (wasmStale) console.error('warning: crates/ changed after packages/core/wasm was built; run `bun run build:wasm` in packages/core');
  return wasmStale;
}
