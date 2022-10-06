<script lang="ts">
    import type { LayoutData } from '$types';
    import '@brainandbones/skeleton/themes/theme-vintage.css';
    import '../app.postcss';
    import { setContextClient } from '$lib/api/urql';
    import { goto, prefetchRoutes } from '$app/navigation';
    import { notifications } from '$lib/components/notifications/notification';
    import { navigating } from '$app/stores';
    import { Spinner } from 'flowbite-svelte';
    import Notifier from '$lib/components/notifications/Notifiers.svelte';

    export let data: LayoutData;

    setContextClient(data.client);

    prefetchRoutes(['/cart']);
</script>

{#if $navigating}
    <Spinner size="10" class="absolute left-[50%] top-[50%]"/>
{/if}

<Notifier></Notifier>

<svelte:head>
    <title>Wgg</title>
</svelte:head>

<slot />
