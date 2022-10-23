<script lang="ts">
    import { CartContentFragment } from '$lib/api/graphql_types';
    import CartProductLine from './CartProductLine.svelte';

    export let data: CartContentFragment[];

    export let columns = ['grid-cols-1'];
</script>

<div class="grid gap-0.5 md:gap-2 {columns.join(' ')}">
    {#each data as content ({ id: content.id, __typename: content.__typename })}
        <CartProductLine data={content} class="w-full" />
    {/each}
</div>
