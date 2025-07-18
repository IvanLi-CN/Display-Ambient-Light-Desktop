import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
// @ts-expect-error - @tailwindcss/vite doesn't have complete TypeScript declarations yet
import tailwindcss from "@tailwindcss/vite";

const mobile =
  process.env.TAURI_PLATFORM === "android" ||
  process.env.TAURI_PLATFORM === "ios";

// https://vitejs.dev/config/
export default defineConfig(() => {

  return {
    plugins: [
      solidPlugin(),
      tailwindcss(),
    ],

    // 优化依赖处理
    optimizeDeps: {
      exclude: ['@tauri-apps/api']
    },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" as const : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        main: 'index.html',
        about: 'about.html'
      }
    }
  },
  };
});
