import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { viteStaticCopy } from "vite-plugin-static-copy";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    tailwindcss(),
    // 复制 floating.html 到构建输出目录
    viteStaticCopy({
      targets: [
        {
          src: "floating.html",
          dest: ".",
        },
      ],
    }),
  ],
  // 防止 Vite 遮盖 Rust 的端口
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
