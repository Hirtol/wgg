import { type Readable, readable, type Subscriber } from 'svelte/store';
import { Unit } from './api/graphql_types';

export const isMobileScreen: Readable<boolean> = readable(false, (set: Subscriber<boolean>) => {
    // Initialise the value
    onResize();
    window.addEventListener('resize', onResize);

    function onResize() {
        set(window.matchMedia('only screen and (max-width: 760px)').matches);
    }

    return () => {
        window.removeEventListener('resize', onResize);
    };
});

export function centsToTextPrice(input: number): string {
    return centsToPrice(input).toFixed(2)
}

export function centsToPrice(input: number): number {
    return input / 100;
}

/**
 * Convert a particular {@link Unit} to a human-readable string, where the first letter is capatilised.
 */
export function unitToText(unit: Unit, short: boolean = false, plural: boolean = true): string {
    // const output = unit.toString();
    let output;
    switch (unit) {
        case Unit.MilliLiter:
            output = short ? 'ml' : 'Milliliter';
            break;
        case Unit.KiloGram:
            output = short ? 'kg' : 'Kilogram';
            break;
        case Unit.Gram:
            output = short ? 'g' : 'Gram';
            break;
        case Unit.Liter:
            output = short ? 'l' : 'Liter';
            break;
        case Unit.Piece:
            output = short ? 'piece' : 'Piece';
            break;
    }

    return output.charAt(0) + output.substring(1).toLocaleLowerCase() + (plural ? '(s)' : '');
}
