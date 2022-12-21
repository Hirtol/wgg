<script lang="ts">
    import PageRoot from '$lib/components/PageRoot.svelte';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import type { PageData } from './$types';

    export let data: PageData;

    $: ({ listStore, cart } = data);
</script>

<PageRoot>
    <HeteroCardList cartStore={cart} data={$listStore.data?.proPromotionsSublist.items ?? []} />
</PageRoot>
