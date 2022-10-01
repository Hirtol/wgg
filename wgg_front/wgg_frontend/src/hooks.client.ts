import type { NavigationEvent } from '@sveltejs/kit';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function handleError({ error, event }: { error: any; event: NavigationEvent }): App.Error {
    console.error('Error', error, event);

    return {
        message: `${event}`,
        code: error.code ?? `Code: ${error}`
    };
}
