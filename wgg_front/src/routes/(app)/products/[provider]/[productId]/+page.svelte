<script lang="ts">
    import type { PageData } from './$types';

    export let data: PageData;

    $: ( {store} = data);
    $: product = $store.data?.proProduct;
</script>

<main class="container mx-auto">
    <section class="card">
        <header>
            <h1>{product?.name}</h1>
        </header>
    </section>
    
</main>
