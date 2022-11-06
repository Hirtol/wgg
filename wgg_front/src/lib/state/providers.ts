import { Provider } from "$lib/api/graphql_types";

/**
 * Return all available providers when this front-end was compiled.
 */
export function getProviders(): Provider[] {
    return Object.values(Provider)
}