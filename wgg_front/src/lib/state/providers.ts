import { Provider, ProviderInfo } from '$lib/api/graphql_types';

export type ProviderMap = Map<Provider, ProviderInfo>;

/**
 * Return all available providers when this front-end was compiled.
 */
export function getProviders(): Provider[] {
    return Object.values(Provider);
}

export function createAvailableProvidersMap(remoteProviders: ProviderInfo[]): ProviderMap {
    return new Map(remoteProviders.map((x) => [x.provider, x]));
}
