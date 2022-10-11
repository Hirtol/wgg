<script lang="ts">
    /**
     * This component is meant to work around [this issue](https://github.com/sveltejs/svelte/issues/7620).
     * Particularly prominent when using Flowbite-Svelte, as they have an unactivated transition delaying all page demounts.
     */

    import { onMount } from 'svelte';
    import { get_current_component } from 'svelte/internal';

    const thisComponent = get_current_component();
    onMount(() => {
        delete thisComponent.$$.fragment.o;
    });
</script>

<slot />
