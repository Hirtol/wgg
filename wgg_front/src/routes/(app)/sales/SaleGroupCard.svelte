<!--
    @component
    A partial Product card dedicated for displaying a Sale Group.
-->
<script lang="ts">
    import { Provider, SaleGroupFragment } from '$lib/api/graphql_types';
    import ProductCardSkeleton from '$lib/components/product_display/products/ProductCardSkeleton.svelte';
    import ProductImage from '$lib/components/product_display/products/ProductImage.svelte';
    import SaleLabel from '$lib/components/product_display/products/SaleLabel.svelte';
    import { salesPageItemUrl } from '$lib/routing';

    export let provider: Provider;
    export let data: SaleGroupFragment;

    $: listUrl = salesPageItemUrl(provider, data.id);
</script>

<ProductCardSkeleton>
    <header class="relative mx-auto">
        <ProductImage data={{ name: data.name, imageUrl: data.imageUrls[0] }} />

        <SaleLabel class="absolute bottom-0 left-0" text={data.saleInfo.label} saleType={data.saleInfo.saleType} />
    </header>

    <div class="body">
        <a class="unstyled text-base font-bold line-clamp-2 md:text-xl" title={data.name} href={listUrl}>{data.name}</a>
    </div>

    <div class="footer flex h-full flex-col justify-end pt-1">
        {#if data.saleDescription}
            <p class="mt-auto line-clamp-2">{@html data.saleDescription}</p>
        {/if}
    </div>
</ProductCardSkeleton>
