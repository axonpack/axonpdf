import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// A relative base, so dist/ works from any subpath of a static host.
export default defineConfig({
  base: './',
  // ledger.jsx also runs in the workers, which have no `window` for fast refresh's footer.
  plugins: [react({ exclude: [/\/ledger\.jsx$/, /\/node_modules\//] })],
  worker: { format: 'es' },
});
