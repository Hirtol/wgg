<script lang="ts">
    import { CartFragment } from '$lib/api/graphql_types';
    import { centsToTextPrice } from '$lib/utils';
    import { Divider } from '@brainandbones/skeleton';

    export let data: CartFragment;
</script>

<div class="min-w-fit">
    <!-- Raw Total Price -->
    <div class="flex flex-row justify-between">
        <h6 class="self-end">Raw Price</h6>
        <h5 class="inline-block">{centsToTextPrice(data.tallies[0].fullPriceCents)}</h5>
    </div>

    <!-- Raw Total Price -->
    <div class="text-wraning-700 flex flex-row justify-between dark:text-warning-600">
        <h6 class="self-end">Discount</h6>
        <h5 class="inline-block">-{centsToTextPrice(data.tallies[0].discountCents)}</h5>
    </div>

    <Divider class="my-2 w-auto" />

    <!-- Raw Total Price -->
    <div class="flex flex-row justify-between">
        <h5 class="self-end">Total</h5>
        <h4 class="inline-block">{centsToTextPrice(data.tallies[0].priceCents)}</h4>
    </div>
</div>
