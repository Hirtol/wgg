// WGG Tailwind Plugin - Theme Colors (from Skeleton Tailwind Plugin)
// - Extends the color palette to accept themeable CSS variables, and most importantly, allow VsCode's typescript plugin to display the colors.
const { default: postcss } = require('postcss');
const plugin = require('tailwindcss/plugin');
const fs = require('fs');

const mode = process.env.NODE_ENV;
const dev = !mode || mode === 'development';

const skeletonThemePath = require.resolve('@brainandbones/skeleton');
const themeUsed = skeletonThemePath + '/../themes/theme-vintage.css';

module.exports = plugin(() => {}, {
    theme: {
        extend: {
            // Extend the colors with the CSS variable values
            // NOTE: Must be RGB to allow for TW opacity value
            colors: {
                ...createColorExtensions()
            }
        }
    }
});

/**
 * Create all color extensions for variables found in a theme file.
 * All variables starting with `--color-` will be converted to Tailwind options.
 * Expects a format similar to this:
 *
 * ```css
 * :root {
 *  --color-primary-50: 255 251 235;
 *  ...
 * }
 * ```
 */
function createColorExtensions() {
    let cssContent = fs.readFileSync(themeUsed);
    let css = postcss.parse(cssContent.toString());
    let colorNodes = css.root().nodes.filter((x) => x.selector == ':root')[0].nodes;
    let nodes = colorNodes.filter((node) => (node.prop ? node.prop.includes('--color-') : false));
    // Split on `-`, get the name of the color variable (e.g, `primary`), then create a unique set
    let keyFields = new Set(nodes.map((item) => item.prop.split('-')[3]));

    let result = {};

    keyFields.forEach((name) => (result[name] = createColorSet(nodes.filter((node) => node.prop.includes(name)))));

    return result;
}

/**
 * @param {import('postcss').ChildNode[]} colorNames
 */
function createColorSet(colorNames) {
    let result = {};

    for (let index = 0; index < colorNames.length; index++) {
        const color = colorNames[index];

        if (color.prop == undefined) {
            continue;
        }

        let lengthValue = color.prop.split('-')[4];

        result[lengthValue] = withOpacityValue(color.prop, color.value);
    }

    return result;
}

/**
 *
 * @param {string} variable The css variable to reference in production
 * @param {string} defaultValue The hex code value which acts as the default color value. Used in development mode to support the VSCode Tailwind plugin with hint colours.
 * @returns The color value with the opacity value appended to it.
 */
function withOpacityValue(variable, defaultValue) {
    return ({ opacityValue }) => {
        const mainVar = `var(${formatHintText(variable, defaultValue)})`;

        if (opacityValue === undefined) {
            return `rgb(${mainVar})`;
        } else {
            return `rgb(${mainVar} / ${opacityValue})`;
        }
    };
}

/**
 * This formats a variable reference with optional fallback in dev mode.
 * When in production mode the fallback is cut out to reduce CSS size.
 *
 * @param {string} variable The css variable to reference in production
 * @param {string} defaultValue The hex code value which acts as the default color value. Used in development mode to support the VSCode Tailwind plugin with hint colours.
 * @returns variable with hint if in dev mode, or just the variable in production
 */
function formatHintText(variable, defaultValue) {
    if (dev) {
        return `${variable}, rgb(${defaultValue})`;
    } else {
        return `${variable}`;
    }
}
