<script lang="ts">
    import ProductImageUnknownSvg from './ProductImageUnknownSvg.svelte';

    export { _class as class };

    export let title: string | undefined = undefined;

    export let blurImage: boolean = false;

    export let data: { imageUrl?: string; name: string };

    let _class: string = '';
</script>

{#if data.imageUrl}
    <img
        src={data.imageUrl}
        draggable="false"
        loading="lazy"
        class="aspect-video h-24 cursor-pointer object-contain md:h-32 {_class}"
        class:opacity-20={blurImage}
        {title}
        alt={data.name} />
{:else}
    <ProductImageUnknownSvg
        class="aspect-video h-24 cursor-pointer object-contain md:h-32 {_class}" />
{/if}
