import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [
    vue()
  ],
  server: {
    port: 5173,
    proxy: {
      "/api": "http://127.0.0.1:8080",
      "/auth": "http://127.0.0.1:8080",
      "/caiji": {
        target: "http://127.0.0.1:8080",
        bypass(req) {
          const url = req.url || "";
          if (
            req.method === "GET" &&
            (url === "/caiji/login" || url === "/caiji/caijiSettings" || url.startsWith("/caiji/caijiSettings/"))
          ) {
            return "/";
          }
        }
      },
      "/app": "http://127.0.0.1:8080"
    }
  }
});
