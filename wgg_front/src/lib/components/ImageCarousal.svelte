<!--
    @component
    A carousal component aimed at displaying product images with thumb-nail preview.
-->
<script lang="ts">
    import { Carousel } from 'flowbite-svelte';
    
    interface ImageInfo {
        id: number;
        imgurl: string;
    }
    export { className as class };

    export let images: ImageInfo[];

    export let showThumbs: boolean = true;

    let className: string = '';
</script>

<div class={className}>
    <Carousel {images} {showThumbs} showCaptions={false} />
</div>
