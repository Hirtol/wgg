<script lang="ts">
    import { Carousel } from 'flowbite-svelte';
    import type { PageData } from './$types';
    import { marked } from 'marked';

    export let data: PageData;

    $: ({ store } = data);
    $: product = $store.data?.proProduct;
    $: images =
        product?.imageUrls.map((url, i) => ({
            id: i,
            imgurl: url
        })) ?? [];

    $: descriptionMarkdown = marked(product?.description ?? '');
</script>

<main class="container mx-auto">
    <section class="card">
        <header>
            <h1>{product?.name}</h1>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-2">
            <div id="descriptionMd">
                {@html descriptionMarkdown}
            </div>

            <Carousel {images} showCaptions={false} showThumbs={false} />
        </div>
    </section>
</main>
