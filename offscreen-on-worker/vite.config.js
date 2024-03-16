import { defineConfig } from 'vite';
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        app: 'static/index.html',
      },
    },
    // Relative to 'root'.
    outDir: '../dist',
  },
  // For getting out of index.html from dist/static directory.
  root: 'static',
  worker: {
    format: 'es',
  },
  plugins: [
    // Makes us be able to use top level await for wasm.
    // Otherwise, we can restrict build.target to 'es2022', which allows top level await.
    wasm(),
    topLevelAwait(),
  ],
  server: {
    port: 8080,
  },
  preview: {
    port: 8080,
  },
});

