import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// A relative base, so dist/ works from any subpath of a static host.
// Cross-origin isolation, so measureUserAgentSpecificMemory can count the workers' memory too.
// public/_headers sends the same two headers from Cloudflare Pages.
const headers = { 'Cross-Origin-Opener-Policy': 'same-origin', 'Cross-Origin-Embedder-Policy': 'require-corp' };

export default defineConfig({
  base: './',
  plugins: [react()],
  worker: { format: 'es' },
  server: { headers },
  preview: { headers },
});
