import { fileURLToPath, URL } from "node:url";
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// For more information see: https://vite.dev/config/
export default defineConfig({
    plugins: [
        vue(),
    ],
    resolve: {
        alias: {
            '@': fileURLToPath(new URL("./src", import.meta.url))
        }
    },
    build: {
        outDir: "../public",
        emptyOutDir: true
    },
    server: {
        proxy: {
        }
    }
})