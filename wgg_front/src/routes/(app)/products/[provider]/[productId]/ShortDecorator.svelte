<script lang="ts">
    import { FullDecoratorFragment } from '$lib/api/graphql_types';

    export { className as class };
    export let data: FullDecoratorFragment[];

    // All the decorators which *might* be rendered by this component
    const renderedItems = ['FreshLabel', 'PrepTime', 'NumberOfServings'];

    let className: string = '';

    $: {
        data.sort((a, b) => a.__typename.localeCompare(b.__typename));
    }
</script>

<div class={className}>
    {#each data as decorator, i}
        {#if decorator.__typename == 'FreshLabel'}
            <span>{decorator.daysFresh}+ day(s) fresh</span>
        {/if}

        {#if decorator.__typename == 'PrepTime'}
            <span>{decorator.timeMinutes} min(s)</span>
        {/if}

        {#if decorator.__typename == 'NumberOfServings'}
            <span>{decorator.amount} serving(s)</span>
        {/if}

        <!-- So long as it isn't the final item add a separator. -->
        {#if i != data.length - 1 && renderedItems.includes(decorator.__typename)}
            <span>|</span>
        {/if}
    {/each}
</div>
