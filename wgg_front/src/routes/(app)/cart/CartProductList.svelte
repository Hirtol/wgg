<script lang="ts">
    import { CartContentFragment } from '$lib/api/graphql_types';
    import { CartStore } from '$lib/state';
    import classNames from 'classnames';
    import CartProductLine from './CartProductLine.svelte';

    export { className as class };
    let className: string = '';

    export let data: CartContentFragment[];
    export let cart: CartStore;

    export let columns = ['grid-cols-1'];

    $: classes = classNames('grid gap-0.5 gap-2 h-fit', columns.join(' '), className);
</script>

<div class={classes}>
    {#each data as content}
        <CartProductLine data={content} class="h-full w-full" {cart} />
    {/each}
</div>
