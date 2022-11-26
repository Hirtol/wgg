<script lang="ts">
    import { Carousel } from 'flowbite-svelte';
    import type { PageData } from './$types';
    import { marked } from 'marked';
    import { Provider, TextType } from '$lib/api/graphql_types';
    import AddComponent from '$lib/components/products/AddComponent.svelte';
    import { getContextClient } from '@urql/svelte';
    import { centsToTextPrice, unitToText } from '$lib/utils';
    import { Divider } from '@brainandbones/skeleton';
    import ImageCarousal from '$lib/components/ImageCarousal.svelte';
    import PriceComponent from '$lib/components/products/PriceComponent.svelte';
    import ShortDecorator from './ShortDecorator.svelte';
    import PriceQuantityComponent from '$lib/components/products/PriceQuantityComponent.svelte';

    export let data: PageData;

    let client = getContextClient();

    $: ({ store, cart } = data);
    $: product = $store.data?.proProduct;

    $: images =
        product?.imageUrls.map((url, i) => ({
            id: i,
            imgurl: url
        })) ?? [];

    $: descrPlainText = product?.description.textType != TextType.Markdown ?? true;
    $: quantity = product ? $cart.getProductQuantity(product.providerInfo.provider, product.id)[0]?.quantity ?? 0 : 0;
    $: daysFresh = product?.decorators.find((x) => x.__typename == 'FreshLabel');
    $: prepTime = product?.decorators.find((x) => x.__typename == 'PrepTime');

    async function updateCartContent(productId: string, provider: Provider, newQuantity: number) {
        await cart.setCartContent({ productId, provider, quantity: newQuantity, __typename: 'RawProduct' }, client);
    }
</script>

{#if product}
    <main class="container mx-auto">
        <section class="card grid grid-cols-1 md:grid-cols-2">
            <ImageCarousal class="card-body" {images} showThumbs={false} />

            <header class="col-span-2 px-4">
                <h2>{product.name}</h2>

                <ShortDecorator
                    class="text-base text-gray-500 line-clamp-1 dark:text-gray-400"
                    data={product.decorators} />

                <PriceQuantityComponent class="!font-normal !text-base" data={product} />

                <PriceComponent
                    data={{
                        displayPrice: product.priceInfo.displayPrice,
                        fullPrice: product.priceInfo.originalPrice
                    }} />

                <AddComponent
                    class="pt-2"
                    normalButton
                    {quantity}
                    on:setQuantity={(e) =>
                        product && updateCartContent(product.id, product.providerInfo.provider, e.detail)} />
            </header>

            <Divider class="col-span-2 mt-4 mb-2" />

            <div class="grid grid-cols-1 px-4 md:grid-cols-2">
                <div id="description" class:whitespace-pre-line={descrPlainText}>
                    {#if !descrPlainText}
                        {@html marked(product.description.text ?? '')}
                    {:else}
                        {product?.description.text ?? ''}
                    {/if}
                </div>
            </div>
        </section>
    </main>
{:else}
    <main class="text-center">
        <h6>Failed to gather product information, please refresh the page.</h6>
    </main>
{/if}

<style>
</style>
