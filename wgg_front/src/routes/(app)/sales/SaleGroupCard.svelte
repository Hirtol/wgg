<!--
    @component
    A partial Product card dedicated for displaying a Sale Group.
-->
<script lang="ts">
    import { Provider, SaleGroupFragment } from '$lib/api/graphql_types';
    import ProductCardSkeleton from '$lib/components/products/ProductCardSkeleton.svelte';
    import ProductImage from '$lib/components/products/ProductImage.svelte';
    import SaleLabel from '$lib/components/products/SaleLabel.svelte';

    export let provider: Provider;
    export let data: SaleGroupFragment;

    $: saleLabel = data.decorators.find((l) => l.__typename == 'SaleLabel');
    $: saleDescription = data.decorators.find((l) => l.__typename == 'SaleDescription');
    $: saleValidity = data.decorators.find((l) => l.__typename == 'SaleValidity');
    $: listUrl = `/sales/${provider}/${data.id}`;
</script>

<ProductCardSkeleton>
    <header class="relative mx-auto">
        <ProductImage data={{ name: data.name, imageUrl: data.imageUrls[0] }} />

        <!-- Sale Label -->
        {#if saleLabel && saleLabel.__typename == 'SaleLabel'}
            <SaleLabel class="absolute bottom-0 left-0 " text={saleLabel.text} />
        {/if}
    </header>

    <div class="body">
        <a class="unstyled text-base font-bold line-clamp-2 md:text-xl" title={data.name} href={listUrl}>{data.name}</a>
    </div>

    <div class="footer h-full flex flex-col justify-end pt-1">
        <p class="line-clamp-2 mt-auto">{saleDescription?.text}</p>
    </div>
</ProductCardSkeleton>
