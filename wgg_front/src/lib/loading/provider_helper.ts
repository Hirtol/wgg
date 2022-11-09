import { Provider } from '$lib/api/graphql_types';
import { notifications } from '$lib/components/notifications/notification';
import { getProviders } from '$lib/state';
import { error } from '@sveltejs/kit';

/**
 * Verify whether the given provider is valid, if it is not (or undefined) then a `401` error is thrown.
 */
export function verifyProviderOrError(provider?: string): Provider {
    if (provider != undefined && getProviders().includes(provider as Provider)) {
        return provider as Provider;
    } else {
        notifications.error(
            `Provider in URL (${provider}) is not valid, please try going back to the main site.`,
            'Invalid Provider',
            5000
        );

        throw error(401, {
            code: '401',
            message: `Invalid Provider: ${provider}`
        });
    }
}
