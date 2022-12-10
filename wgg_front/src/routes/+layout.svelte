<script lang="ts">
    import '../theme.postcss';
    import '../app.postcss';
    import { afterNavigate, beforeNavigate, preloadCode } from '$app/navigation';
    import { setContextClient } from '$lib/api/urql';
    import GlobalLoading from '$lib/components/global_progress/GlobalLoading.svelte';
    import { globalLoading } from '$lib/components/global_progress/global_loading';
    import Notifier from '$lib/components/notifications/Notifiers.svelte';
    import type { LayoutData } from './$types';
    import { cartPageRootUrl } from '$lib/routing';

    export let data: LayoutData;

    setContextClient(data.client);

    preloadCode(cartPageRootUrl);

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
