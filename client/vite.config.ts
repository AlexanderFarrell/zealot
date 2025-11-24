import { defineConfig } from 'vite';

export default defineConfig({
    server: {
        port: 5173,
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:8082',
                changeOrigin: true,
                rewrite: (path) => path.replace(/^\/api/, ''),
                configure: (proxy: any) => {
                    // optional: quick logs to see what's happening
                    proxy.on('error', (err:any, req:any) => {
                        console.error('Proxy error:', err?.message, req?.url);
                    });
                    proxy.on('proxyReq', (_proxyReq:any, req:any) => {
                        console.log('→', req.method, req.url);
                    });
                    proxy.on('proxyRes', (proxyRes:any, req:any) => {
                        console.log('←', proxyRes.statusCode, req.url);
                    });
                },
            }
        }
    }
})