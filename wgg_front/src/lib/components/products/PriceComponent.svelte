<script lang="ts">
    import { centsToTextPrice } from '$lib/utils';
    import classNames from 'classnames';

    export { classes as class };
    export let data: { displayPrice: number; fullPrice: number };

    export let dashed: boolean = false;

    let classes = '';

    $: classLocal = classNames(classes, 'block');
    $: hasSale = data.displayPrice == data.fullPrice;
    $: displayPrice = centsToTextPrice(data.displayPrice);
    $: fullPrice = centsToTextPrice(data.fullPrice);
</script>

<div class={classLocal}>
    <h3 class:line-through={dashed}>
        {#if !hasSale}
            <span class="inline-block text-gray-500 line-through dark:text-gray-400">{fullPrice}</span>
        {/if}

        <span
            class="inline-block"
            class:text-warning-700={!hasSale}
            class:dark:text-warning-600={!hasSale}
            class:line-through={dashed}>
            {displayPrice}
        </span>
    </h3>
</div>
