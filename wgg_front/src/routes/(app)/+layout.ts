import { authenticateUser } from '$lib/user';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async (event) => {
    const { client } = await event.parent();
    const isAuthenticated = await authenticateUser(client);

    // Perform authentication check
    if (!isAuthenticated && !event.url.pathname.includes('login')) {
        const loginUrl = event.url.href.length > 0 ? `/login?redirect=${event.url.pathname}` : '/login';

        throw redirect(302, loginUrl);
    }
};
