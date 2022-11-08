<!--
    @component
    List of (Hetero)CardLists enhanced with partial fill-in.
-->
<script lang="ts">
    import { ProductCardFragment, SaleCategoryFragment } from '$lib/api/graphql_types';
    import ProductCardSkeleton from '$lib/components/products/ProductCardSkeleton.svelte';
    import ProductList from '$lib/components/products/ProductList.svelte';
    import { CartStore } from '$lib/state';
    import { ChevronRight } from 'carbon-icons-svelte';
    import HeteroCardList from './HeteroCardList.svelte';

    export let cart: CartStore;
    export let data: SaleCategoryFragment[];

    export let totalCount: number;

    /**
     * Typescript hack to ensure we get the correct type.
     */
    function getSearchProducts(items: any[]): ProductCardFragment[] {
        return items as ProductCardFragment[];
    }
</script>

<ul>
    {#each data as sale (sale.name)}
        <li>
            <h5>{sale.name}</h5>
            <HeteroCardList cartStore={cart} data={sale}>
                <!-- More button to get full list. -->
                <svelte:fragment slot="last-grid">
                    {#if !sale.complete}
                        <ProductCardSkeleton class="min-h-[10rem]">
                            <button class="btn flex h-full w-full flex-col items-center justify-center" on:click={() => {}}>
                                <h4>More</h4>
                                <span
                                    class="btn-icon inline-flex !w-9 items-center justify-center rounded-full bg-primary-400 !p-0 dark:bg-primary-800">
                                    <ChevronRight size={24} />
                                </span>
                            </button>
                        </ProductCardSkeleton>
                    {/if}
                </svelte:fragment>
            </HeteroCardList>
        </li>
    {/each}
</ul>
