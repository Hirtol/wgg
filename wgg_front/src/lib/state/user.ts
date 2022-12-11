import { derived, type Readable, writable, type Writable } from 'svelte/store';
import {
    type ViewerContextFragment,
    ViewerInfoQueryDocument,
    SubmitLoginDocument,
    LogoutMutationDocument
} from '$lib/api/graphql_types';
import { asyncMutationStore, asyncQueryStore, Client } from '$lib/api/urql';

type isAuthenticated = boolean;
// Current cart is an implementation detail for the GraphQL fetching in one round-trip, not relevant on actual user.
export type UserData = Omit<ViewerContextFragment, 'currentCart'>;
export type UserStore = Writable<UserData | undefined>;

/**
 * The current session
 */
export const authSession: UserStore = writable(undefined);
export const isUserAuthenticated: Readable<isAuthenticated> = derived(authSession, (val) => val != undefined);

export async function loginUser(
    email: string,
    password: string,
    client: Client
): Promise<{ item: UserData | undefined }> {
    const { item } = await asyncMutationStore({
        query: SubmitLoginDocument,
        variables: { email, password },
        client
    });

    if (item.error) {
        console.log('Failed to log-in user', item.error);
    }

    authSession.set(item.data?.login.user);

    return {
        item: item.data?.login.user
    };
}

export async function logoutUser(client: Client): Promise<void> {
    const _ = await asyncMutationStore({ query: LogoutMutationDocument, client });

    authSession.set(undefined);
}
