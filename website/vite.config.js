import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        blog: resolve(__dirname, 'blog/index.html'),
        'blog-why-vais': resolve(__dirname, 'blog/why-vais.html'),
      },
    },
  },
  server: {
    port: 3001,
    open: true,
  },
});
