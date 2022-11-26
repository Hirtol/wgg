<script lang="ts">
    import { Unit } from '$lib/api/graphql_types';
    import { centsToTextPrice, unitToText } from '$lib/utils';
    import classNames from 'classnames';

    interface DataRequired {
        unitQuantity: { amount: number; unit: Unit };
        priceInfo: { unitPrice?: { price: number; unit: Unit } };
    }

    export { classes as class };
    export let data: DataRequired;

    let classes = '';

    $: classLocal = classNames("text-sm flex text-gray-500 line-clamp-1 dark:text-gray-400", classes);
</script>

<h6 class={classLocal}>
    {data.unitQuantity.amount}
    {unitToText(data.unitQuantity.unit, true, false)}

    {#if data.priceInfo.unitPrice}
        {@const unitPriceCents = centsToTextPrice(data.priceInfo.unitPrice.price)}
        {@const unitText = unitToText(data.priceInfo.unitPrice.unit, true, false)}
        €{unitPriceCents}/{unitText}
    {/if}
</h6>
