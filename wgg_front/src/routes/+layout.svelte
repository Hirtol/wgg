<script lang="ts">
    import type { LayoutData } from './$types';
    import '@brainandbones/skeleton/themes/theme-vintage.css';
    import '../app.postcss';
    import { setContextClient } from '$lib/api/urql';
    import { afterNavigate, beforeNavigate, goto, prefetchRoutes } from '$app/navigation';
    import { notifications } from '$lib/components/notifications/notification';
    import { navigating, page } from '$app/stores';
    import { Spinner } from 'flowbite-svelte';
    import Notifier from '$lib/components/notifications/Notifiers.svelte';
    import GlobalLoading from '$lib/components/global_progress/GlobalLoading.svelte';
    import { globalLoading } from '$lib/components/global_progress/global_loading';

    export let data: LayoutData;

    setContextClient(data.client);

    prefetchRoutes(['/cart']);

    beforeNavigate((nav) => {
        globalLoading.start();
    });

    afterNavigate((nav) => {
        globalLoading.stop();
    });
</script>

<svelte:head>
    <title>Wgg</title>
</svelte:head>

<!-- Shows a loader bar at the top of the page similar to Github -->
<GlobalLoading />
<!-- Shows notifications for the user -->
<Notifier />

<slot />
