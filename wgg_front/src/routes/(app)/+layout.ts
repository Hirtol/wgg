import { authenticateUser, authSession } from '$lib/state';
import { redirect } from '@sveltejs/kit';
import { get } from 'svelte/store';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async (event) => {
    // First check if we just had a login event, if not we proceed with the manual check.
    if (get(authSession) == undefined) {
        const { client } = await event.parent();
        const { isAuthenticated } = await authenticateUser(client);

        // Perform authentication check
        if (!isAuthenticated) {
            const loginUrl = event.url.href.length > 0 ? `/login?redirect=${event.url.pathname}` : '/login';

            throw redirect(302, loginUrl);
        }
    }
};
