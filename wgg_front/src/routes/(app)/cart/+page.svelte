<script lang="ts">
    import { page } from '$app/stores';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import type { PageData } from './$types';
    import CartProductList from './CartProductList.svelte';

    export let data: PageData;

    $: ({ cart } = $page.data);
</script>

<PageRoot>
    <p>Cart Page</p>
    <CartProductList data={$cart?.contents ?? []}></CartProductList>
</PageRoot>
