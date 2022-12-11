import { getContext, setContext } from 'svelte';
import { Unit } from './api/graphql_types';
import { CartStore } from './state';

const CART_CONTEXT_KEY = 'cart';

export function setContextCart(cart: CartStore) {
    setContext(CART_CONTEXT_KEY, cart);
}

export function getContextCart(): CartStore {
    return getContext(CART_CONTEXT_KEY);
}

export function capitaliseFirst(input: string): string {
    return input.charAt(0).toUpperCase() + input.substring(1).toLocaleLowerCase();
}

export function centsToTextPrice(input: number): string {
    return centsToPrice(input).toFixed(2);
}

export function centsToPrice(input: number): number {
    return input / 100;
}

/**
 * Convert a particular {@link Unit} to a human-readable string, where the first letter is capitilised.
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

    return capitaliseFirst(output) + (plural ? '(s)' : '');
}
