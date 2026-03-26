import { defineConfig } from "vite";

export default defineConfig({
    server: {
        host: true,
        proxy: {
            "/api": {
                target: "http://127.0.0.1:8456",
                changeOrigin: true,
                rewrite: (path) => path.replace(/^\/api/, ""),
            },
        },
    },
});
