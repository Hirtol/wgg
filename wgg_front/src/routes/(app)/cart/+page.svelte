<script lang="ts">
    import { page } from '$app/stores';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import type { PageData } from './$types';
    import CartOverview from './CartOverview.svelte';
    import CartProductList from './CartProductList.svelte';

    export let data: PageData;

    $: cart = data.result;
</script>

<PageRoot>
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        <CartProductList columns={['grid-cols-1', '2xl:grid-cols-2']} data={$cart.data?.cartCurrent.contents ?? []} />

        <div class="card h-min p-4">
            <h4 class="text-center">Overview</h4>
            {#if $cart.data}
                <CartOverview data={$cart.data?.cartCurrent} />
            {/if}
        </div>
    </div>
</PageRoot>
