<script lang="ts">
    import { centsToTextPrice } from '$lib/utils';

    export { _class as class };

    export let data: { displayPrice: number; originalPrice: number };

    export let dashed: boolean = false;

    let _class = '';

    $: hasSale = data.displayPrice == data.originalPrice;
</script>

<div class="block {_class}">
    <h3 class:line-through={dashed}>
        {#if !hasSale}
            <span class="inline-block text-gray-500 line-through dark:text-gray-400"
                >{centsToTextPrice(data.originalPrice)}</span>
        {/if}

        <span
            class="inline-block"
            class:text-warning-700={!hasSale}
            class:dark:text-warning-600={!hasSale}
            class:line-through={dashed}>
            {centsToTextPrice(data.displayPrice)}
        </span>
    </h3>
</div>
