import { defineConfig } from 'vite';
import fs from 'node:fs';
import monacoEditorPlugin from 'vite-plugin-monaco-editor';

function monacoNodeCompatPlugin() {
  const originalRmdirSync = fs.rmdirSync;

  return {
    name: 'monaco-node-compat',
    enforce: 'pre',
    configResolved() {
      if (fs.rmdirSync.__vaisMonacoNodeCompat) return;

      const patchedRmdirSync = function patchedRmdirSync(path, options) {
        if (options?.recursive) {
          return fs.rmSync(path, {
            recursive: true,
            force: options.force ?? false,
          });
        }
        return originalRmdirSync.apply(this, arguments);
      };
      patchedRmdirSync.__vaisMonacoNodeCompat = true;
      fs.rmdirSync = patchedRmdirSync;
    },
  };
}

export default defineConfig({
  base: process.env.NODE_ENV === 'production' ? '/playground/' : '/',
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
    monacoNodeCompatPlugin(),
    monacoEditorPlugin.default({
      languageWorkers: ['editorWorkerService']
    })
  ]
});
