import { defineConfig } from 'vite';
import monacoEditorPlugin from 'vite-plugin-monaco-editor';

export default defineConfig({
  base: '/',
  server: {
    port: 3000,
    open: true
  },
  build: {
    outDir: 'dist',
    sourcemap: true
  },
  optimizeDeps: {
    exclude: []
  },
  plugins: [
    monacoEditorPlugin.default({
      languageWorkers: ['editorWorkerService']
    })
  ]
});
