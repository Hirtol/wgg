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
    import { Modal, modalStore } from '@skeletonlabs/skeleton';
    import { page } from '$app/stores';
    import { setContextPreference } from '$lib/state';

    export let data: LayoutData;

    setContextClient(data.client);
    setContextPreference(data.preferences);

    preloadCode(cartPageRootUrl);

    beforeNavigate((nav) => {
        // First close the modals if there are any.
        if ($modalStore.length > 0) {
            modalStore.clear();
            nav.cancel();
        } else {
            globalLoading.start();
        }
    });

    afterNavigate((nav) => {
        globalLoading.stop();
    });
</script>

<svelte:head>
    <title>{$page.data.title}</title>
</svelte:head>

<!-- Shows a loader bar at the top of the page similar to Github -->
<GlobalLoading />
<!-- Shows notifications for the user -->
<Notifier />
<!-- Shows modals -->
<Modal width="w-full max-w-2xl md:max-w-5xl overscroll-none" height="h-auto max-h-full flex flex-col" />

<slot />
