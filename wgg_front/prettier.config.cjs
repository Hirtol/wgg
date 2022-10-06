module.exports = {
    trailingComma: 'none',
    tabWidth: 4,
    singleQuote: true,
    bracketSameLine: true,
    useTabs: false,
    printWidth: 120,
    plugins: ['prettier-plugin-tailwindcss'],
    overrides: [
        {
            files: '*.svelte',
            options: { parser: 'svelte' }
        }
    ]
};
