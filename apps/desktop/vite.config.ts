import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
    server: {
        port: 5175,
        strictPort: true,
        host: host || false,
        hmr: host
            ? { protocol: 'ws', host, port: 5175 }
            : undefined,
    },
    envPrefix: ['VITE_', 'TAURI_ENV_*'],
    build: {
        outDir: 'dist',
        emptyOutDir: true,
    },
});
