<script lang="ts">
    import { centsToTextPrice } from '$lib/utils';
    import { Divider } from '@skeletonlabs/skeleton';

    export let data: { fullPriceCents: number; discountCents: number; priceCents: number };
</script>

<div class="min-w-fit">
    <!-- Raw full price -->
    <div class="flex flex-row justify-between">
        <h6 class="self-end">Raw Price</h6>
        <h5 class="inline-block">{centsToTextPrice(data.fullPriceCents)}</h5>
    </div>

    <!-- Raw discount -->
    <div class="flex flex-row justify-between text-warning-700 dark:text-warning-600">
        <h6 class="self-end">Discount</h6>
        <h5 class="inline-block">-{centsToTextPrice(data.discountCents)}</h5>
    </div>

    <Divider class="my-2 w-auto" />

    <!-- Total price -->
    <div class="flex flex-row justify-between">
        <h5 class="self-end">Total</h5>
        <h4 class="inline-block">{centsToTextPrice(data.priceCents)}</h4>
    </div>
</div>
