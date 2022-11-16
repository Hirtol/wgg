import { Provider } from './api/graphql_types';

export const loginPageRootUrl = 'login';
export const salesPageRootUrl = 'sales';
export const productPageRootUrl = 'products';
export const cartPageRootUrl = 'cart';
export const aggregatePageRootUrl = 'aggregates';

export const productPageItemUrl = (provider: Provider, productId: string) =>
    `${productPageRootUrl}/${provider}/${productId}`;

export const salesPageItemUrl = (provider: Provider, listId: string) => `${salesPageRootUrl}/${provider}/${listId}`;
