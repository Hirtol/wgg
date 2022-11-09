<!-- 
    @component
    Provides a standardised grid layout to display Product-like Cards in.

    On the primary slot exports the following props:
    * `updateCartContent` - An async function to call whenever one wants to change the current cart's contents.
 -->
<script lang="ts">
    import { Provider } from '$lib/api/graphql_types';
    import { getContextClient } from '$lib/api/urql';
    import { CartStore } from '$lib/state';

    export let cartStore: CartStore;

    export let columns = ['grid-cols-2', 'md:grid-cols-3', 'lg:grid-cols-4', 'xl:grid-cols-5', '2xl:grid-cols-6'];

    const client = getContextClient();

    /**
     * Update the given productId to be the new quantity.
    */
    async function updateCartContent(
        event: CustomEvent<{
            productId: string;
            provider: Provider;
            newQuantity: number;
        }>
    ) {
        let { productId, provider, newQuantity } = event.detail;
        await cartStore.setCartContent(
            { productId, provider, quantity: newQuantity, __typename: 'RawProduct' },
            client
        );
    }
</script>

<div class="grid gap-0.5 md:gap-2 {columns.join(' ')}">
    <slot {updateCartContent} />
</div>
