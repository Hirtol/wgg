const config = {
    content: [
        './src/**/*.{html,js,svelte,ts}',
        './node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}',
        './node_modules/@skeletonlabs/skeleton/**/*.{html,js,svelte,ts}'
    ],
    darkMode: 'class',

    plugins: [
        require('flowbite/plugin'),
        require('./src/lib/tailwind/theme.cjs'),
        require('@tailwindcss/forms'),
        require('@tailwindcss/line-clamp')
    ]
};

module.exports = config;
