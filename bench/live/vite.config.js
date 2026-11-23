import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// A relative base, so dist/ works from any subpath of a static host.
export default defineConfig({
  base: './',
  plugins: [react()],
  worker: { format: 'es' },
});
