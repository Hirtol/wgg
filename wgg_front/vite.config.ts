import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';

const config: UserConfig = {
    plugins: [sveltekit()],
    server: {
        proxy: {
            '/api': {
                target: 'http://localhost:9000',
                changeOrigin: true
            }
        },
        port: 3000
    },
    optimizeDeps: {
        exclude: ['@urql/svelte', '@urql/exchange-graphcache', '@urql/exchange-request-policy']
    }
};

export default config;
