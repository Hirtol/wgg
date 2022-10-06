import { derived, Readable, writable, Writable } from 'svelte/store';
import { ViewerContextFragment, ViewerInfoQueryDocument } from '$lib/api/graphql_types';
import { Client } from '$lib/api/urql';

type isAuthenticated = boolean;

/**
 * The current session
 */
export const authSession: Writable<ViewerContextFragment | undefined> = writable(undefined);
export const isUserAuthenticated: Readable<isAuthenticated> = derived(authSession, (val) => val !== null);

/**
 * Authenticate the user, if possible.
 *
 * If the current user is authenticated, the `authSession` and `isUserAuthenticated` stores will be initialised.
 */
export async function authenticateUser(client: Client): Promise<isAuthenticated> {
    const authContext = await client.query(ViewerInfoQueryDocument, {}).toPromise();

    if (authContext.error) {
        console.log('User is not authenticated', authContext.error);
        authSession.set(undefined);
    } else {
        authSession.set(authContext.data?.viewer);
    }
    console.log(authContext.data != undefined, authContext);

    return authContext.data != undefined;
}
