import { authenticateUser, authSession, initialiseRealCart } from '$lib/state';
import { redirect } from '@sveltejs/kit';
import { get } from 'svelte/store';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async (event) => {
    // First check if we just had a login event, if not we proceed with the manual check.
    const { client } = await event.parent();
    let cartContents = undefined;
    
    if (get(authSession) == undefined) {
        const { isAuthenticated, user } = await authenticateUser(client);

        // Perform authentication check
        if (!isAuthenticated) {
            const loginUrl = event.url.href.length > 0 ? `/login?redirect=${event.url.pathname}` : '/login';

            throw redirect(302, loginUrl);
        } else {
            cartContents = user?.currentCart;
        }
    }

    // We can safely assume we're authenticated.
    return {
        cart: initialiseRealCart(client, cartContents)
    }
};
