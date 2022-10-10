// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces

import { CartStore } from '$lib/state';

// and what to do when importing types
declare global {
    namespace App {
        // interface Locals {}
        interface PageData {
            cart: CartStore;
        }
        interface Error {
            message: string;
            code: string;
        }
        // interface Platform {}
    }
}
