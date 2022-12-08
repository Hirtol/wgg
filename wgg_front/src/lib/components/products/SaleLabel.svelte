<!--
    @component
    The sale label component.

    If `saleType` is `undefined` this will show a warning to the user that this sale wasn't properly parsed.
-->
<script lang="ts">
    import { SaleTypeNameFragment } from '$lib/api/graphql_types';
    import { Warning } from 'carbon-icons-svelte';
    import classNames from 'classnames';

    export { className as class };
    export let text: string;
    export let saleType: SaleTypeNameFragment | undefined = undefined;

    let className: string = '';
</script>

<span class={classNames('badge w-min bg-primary-300 dark:bg-primary-800', className)}>{text}</span>

{#if !saleType}
    <Warning
        class="inline"
        color="red"
        title="Couldn't parse sale type, cart tally will be wrong if this product is included!" />
{/if}
