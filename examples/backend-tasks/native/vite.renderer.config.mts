import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath, URL } from 'node:url';
export default defineConfig({ plugins: [react(), tailwindcss()],
  server: { host: '127.0.0.1', port: 5183, strictPort: true, fs: { allow: [fileURLToPath(new URL('..', import.meta.url))] } },
  resolve: { preserveSymlinks: false, alias: { '@': fileURLToPath(new URL('../web/src', import.meta.url)) },
    dedupe: ['react', 'react-dom', '@tanstack/react-query', '@tanstack/query-core'] },
});
