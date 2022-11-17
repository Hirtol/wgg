<script lang="ts">
    import { Carousel } from 'flowbite-svelte';
    import type { PageData } from './$types';
    import { marked } from 'marked';
    import { Provider, TextType } from '$lib/api/graphql_types';
    import AddComponent from '$lib/components/products/AddComponent.svelte';
    import { getContextClient } from '@urql/svelte';

    export let data: PageData;

    let client = getContextClient();

    $: ({ store, cart } = data);
    $: product = $store.data?.proProduct;
    $: images =
        product?.imageUrls.map((url, i) => ({
            id: i,
            imgurl: url
        })) ?? [];
    $: isPlainText = product?.description.textType != TextType.Markdown ?? true;
    $: quantity = product ? $cart.getProductQuantity(product.providerInfo.provider, product.id)[0]?.quantity ?? 0 : 0;

    async function updateCartContent(productId: string, provider: Provider, newQuantity: number) {
        await cart.setCartContent(
            { productId, provider, quantity: newQuantity, __typename: 'RawProduct' },
            client
        );
    }
</script>

<main class="container mx-auto">
    <section class="card">
        <header>
            <h1>{product?.name}</h1>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-2">
            <AddComponent normalButton {quantity} on:setQuantity={(e) => product && updateCartContent(product.id, product.providerInfo.provider, e.detail)} />
            <div id="description" class:whitespace-pre-line={isPlainText}>
                {#if !isPlainText}
                    {@html marked(product?.description.text ?? '')}
                {:else}
                    {product?.description.text ?? ''}
                {/if}
            </div>

            <Carousel {images} showCaptions={false} showThumbs={false} />
        </div>
    </section>
</main>
