<script lang="ts">
    import { centsToTextPrice } from '$lib/utils';
    import classNames from 'classnames';

    export let data: { displayPrice: number; fullPrice: number };

    $: classes = classNames($$restProps.class, 'block');
    $: hasSale = data.displayPrice == data.fullPrice;
    $: displayPrice = centsToTextPrice(data.displayPrice);
    $: fullPrice = centsToTextPrice(data.fullPrice);
</script>

<div class={classes}>
    <h4>
        {#if !hasSale}
            <span class="inline-block text-gray-500 line-through dark:text-gray-400">{fullPrice}</span>
        {/if}

        <span class="inline-block" class:text-warning-700={!hasSale} class:dark:text-warning-600={!hasSale}>
            {displayPrice}
        </span>
    </h4>
</div>
