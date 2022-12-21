// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces

// and what to do when importing types
namespace App {
    // interface Locals {}
    interface PageData {
        // Needed as adding individual <svelte:head> elements to each page doesn't work when opening a new tab of our site (e.g, middle-mouse click a link)
        // Only the root <svelte:head> will then be used until you navigate.
        title: string
    }
    interface Error {
        message: string;
        code: string;
    }
    // interface Platform {}
}
