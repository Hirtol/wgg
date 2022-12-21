<script lang="ts">
    import { Unit } from '$lib/api/graphql_types';
    import { centsToTextPrice, unitToText } from '$lib/utils';

    interface DataRequired {
        unitQuantity: { amount: number; unit: Unit };
        priceInfo: { unitPrice?: { price: number; unit: Unit } };
    }

    export { _class as class };
    export let data: DataRequired;

    let _class = '';
</script>

<h6 class="text-sm flex text-gray-500 line-clamp-1 dark:text-gray-400 {_class}">
    {data.unitQuantity.amount}
    {unitToText(data.unitQuantity.unit, true, false)}

    {#if data.priceInfo.unitPrice}
        {@const unitPriceCents = centsToTextPrice(data.priceInfo.unitPrice.price)}
        {@const unitText = unitToText(data.priceInfo.unitPrice.unit, true, false)}
        €{unitPriceCents}/{unitText}
    {/if}
</h6>
