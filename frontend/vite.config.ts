import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Build output goes to frontend/dist, which the Rust backend embeds.
// In dev, /api and /start.sh are proxied to the local Rust server.
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      '/api': 'http://localhost:8090',
      '/start.sh': 'http://localhost:8090',
    },
  },
});
