import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';
import { optimizeImports } from 'carbon-preprocess-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: [
        preprocess({
            postcss: true
        }),
        optimizeImports()
    ],
    kit: {
        adapter: adapter({
            fallback: 'index.html'
        })
    }
};

export default config;
