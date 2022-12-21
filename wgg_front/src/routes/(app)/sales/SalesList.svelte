<!--
    @component
    List of (Hetero)CardLists.
-->
<script lang="ts">
    import { SaleCategoryFragment } from '$lib/api/graphql_types';
    import HeteroCardList from '$lib/components/product_display/HeteroCardList.svelte';
    import ProductCardSkeleton from '$lib/components/product_display/products/ProductCardSkeleton.svelte';
    import { salesPageItemUrl } from '$lib/routing';
    import { CartStore } from '$lib/state';
    import { ChevronRight } from 'carbon-icons-svelte';

    export let cart: CartStore;
    export let data: SaleCategoryFragment[];
</script>

<ul>
    {#each data as sale (sale.name)}
        <li>
            <h5>{sale.name}</h5>
            <HeteroCardList cartStore={cart} data={sale.items}>
                <!-- More button to get full list. -->
                <svelte:fragment slot="last-grid">
                    {#if !sale.complete && sale.id}
                        {@const moreUrl = salesPageItemUrl(sale.providerInfo.provider, sale.id)}

                        <ProductCardSkeleton class="min-h-[14rem] md:min-h-[18rem]">
                            <a
                                class="btn unstyled flex h-full w-full flex-col items-center justify-center"
                                href={moreUrl}
                                title="More">
                                <h4>More</h4>
                                <span
                                    class="btn-icon inline-flex !w-9 items-center justify-center rounded-full bg-primary-400 !p-0 dark:bg-primary-800">
                                    <ChevronRight size={24} />
                                </span>
                            </a>
                        </ProductCardSkeleton>
                    {/if}
                </svelte:fragment>
            </HeteroCardList>
        </li>
    {/each}
</ul>
