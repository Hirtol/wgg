// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces

import { UserStore } from '$lib/user';

// and what to do when importing types
declare global {
    namespace App {
        // interface Locals {}
        interface PageData {
            user: UserStore;
        }
        interface Error {
            message: string;
            code: string;
        }
        // interface Platform {}
    }
}
