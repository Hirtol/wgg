<script lang="ts">
    import { CartContentFragment } from '$lib/api/graphql_types';
    import classNames from 'classnames';
    import CartProductLine from './CartProductLine.svelte';

    export { className as class };
    let className: string;

    export let data: CartContentFragment[];

    export let columns = ['grid-cols-1'];

    $: classes = classNames('grid gap-0.5 md:gap-2', columns.join(' '), className);
</script>

<div class={classes}>
    {#each data as content ({ id: content.id, __typename: content.__typename })}
        <CartProductLine data={content} class="w-full" />
    {/each}
</div>
