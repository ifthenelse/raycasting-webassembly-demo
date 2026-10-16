import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  root: '.',
  // relative base so the build works under the GitHub Pages project subpath
  base: './',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    open: '/index.html',
  },
});
