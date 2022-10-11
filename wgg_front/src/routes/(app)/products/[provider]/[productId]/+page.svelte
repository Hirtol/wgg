<script lang="ts">
    import { Carousel } from 'flowbite-svelte';
    import type { PageData } from './$types';
    import { marked } from 'marked';
    import { TextType } from '$lib/api/graphql_types';

    export let data: PageData;

    $: ({ store } = data);
    $: product = $store.data?.proProduct;
    $: images =
        product?.imageUrls.map((url, i) => ({
            id: i,
            imgurl: url
        })) ?? [];
    $: isPlainText = product?.description.textType != TextType.Markdown ?? true;
</script>

<main class="container mx-auto">
    <section class="card">
        <header>
            <h1>{product?.name}</h1>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-2">
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
