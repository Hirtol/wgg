<script lang="ts">
    import { page } from '$app/stores';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import type { PageData } from './$types';
    import CartProductList from './CartProductList.svelte';

    export let data: PageData;

    $: cart = data.result;
</script>

<PageRoot>
    <CartProductList data={$cart.data?.cartCurrent.contents ?? []}></CartProductList>
</PageRoot>
