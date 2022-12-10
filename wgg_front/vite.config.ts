import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';

function manualChunks(id: string) {
    if (id.includes('node_modules') && !id.includes('@sveltejs')) {
        return 'vendor';
    }
}

const config: UserConfig = {
    plugins: [sveltekit()],
    server: {
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:9000',
                changeOrigin: true
            }
        },
        port: 3000
    },
    optimizeDeps: {
        exclude: ['@urql/svelte', '@urql/exchange-graphcache', '@urql/exchange-request-policy']
    },
    build: {
        rollupOptions: {
            output: {
                manualChunks: manualChunks
            }
        }
    }
};

export default config;
