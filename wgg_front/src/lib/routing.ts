import { goto } from '$app/navigation';
import { Provider } from './api/graphql_types';

export const loginPageRootUrl = 'login';
export const salesPageRootUrl = 'sales';
export const productPageRootUrl = 'products';
export const cartPageRootUrl = 'cart';
export const aggregatePageRootUrl = 'aggregates';

export const productPageItemUrl = (provider: Provider, productId: string) =>
    `/${productPageRootUrl}/${provider}/${productId}`;

export const salesPageItemUrl = (provider: Provider, listId: string) => `${salesPageRootUrl}/${provider}/${listId}`;

/**
 * Update a query parameter, and goto the new page to update browser history.
 */
export async function updateQueryParameter(currentUrl: URL, key: string, value: string, opts?: { replaceState: boolean }) {
    const newUrl = new URL(currentUrl);
    newUrl.searchParams.set(key, value);
    if (currentUrl.searchParams.get(key) != newUrl.searchParams.get(key)) {
        await goto(newUrl, opts);
    }
}