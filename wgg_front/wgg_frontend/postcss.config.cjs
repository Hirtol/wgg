const tailwindcss = require('tailwindcss');
const autoprefixer = require('autoprefixer');
const postcssLightningcss = require('postcss-lightningcss');

const mode = process.env.NODE_ENV;
const dev = mode === 'development';

const config = {
    plugins: [
        //Some plugins, like tailwindcss/nesting, need to run before Tailwind,
        tailwindcss(),
        //But others, like autoprefixer, need to run after,
        autoprefixer,
        !dev &&
            postcssLightningcss({
                // Use a browserslist query that will inform which browsers are supported
                // Will add or remove vendor prefixes that are needed or not anymore
                // browsers: '>= .25%',

                // https://www.npmjs.com/package/lightningcss#user-content-documentation
                lightningcssOptions: {
                    // Enable minification (default: true)
                    minify: true,
                    // Ignore unsupported rules.
                    errorRecovery: true,
                    // Enable source maps (default: true)
                    sourceMap: true
                }
            })
    ]
};

module.exports = config;
