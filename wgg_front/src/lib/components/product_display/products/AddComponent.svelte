<script lang="ts">
    import { Add, Subtract } from 'carbon-icons-svelte';
    import { createEventDispatcher } from 'svelte';
    import { fly } from 'svelte/transition';

    export let normalButton: boolean = false;

    /**
     * Non-Bindable property which declares the current quantity available
     */
    export let quantity: number;

    /**
     * Whether the current set of buttons should be permanently expanded to have both the add and subtract buttons,
     * or just the quantity button.
     */
    export let permanentlyExpanded: boolean = false;

    /**
     * How long the buttons should stay expanded when pressed.
     */
    export let expandedDuration: number = 2000;

    /**
     * The title of the product, used as a title for the controls.
    */
    export let productTitle: string = '';

    /**
     * Whether the current set of buttons should be expanded to have both the add and subtract buttons,
     * or just the quantity button.
     */
    let isExpanded: boolean = permanentlyExpanded;

    let timeoutId: number | undefined;

    const dispatch = createEventDispatcher<{ setQuantity: number }>();

    function setExpanded() {
        // On refresh calls.
        if (timeoutId != undefined) {
            clearTimeout(timeoutId);
        }

        isExpanded = true;
        timeoutId = window.setTimeout(() => (isExpanded = false), expandedDuration);
    }

    function setQuantity(quantityNew: number) {
        if (!permanentlyExpanded) {
            setExpanded();
        }

        dispatch('setQuantity', quantityNew);
    }
</script>

<div {...$$restProps}>
    {#if quantity > 0}
        {#if isExpanded}
            <div class="flex flex-nowrap justify-between gap-0.5" in:fly>
                <button
                    title="Subtract Quantity {productTitle}"
                    class="btn-icon btn-filled-primary !w-6 rounded-full !p-0"
                    on:click={() => setQuantity(--quantity)}>
                    <Subtract size={24} />
                </button>
                <button
                    title="Hide"
                    class="btn-icon btn-filled-primary !w-6 rounded-full !p-0"
                    on:click={() => (isExpanded = permanentlyExpanded)}>
                    {quantity}
                </button>
                <button
                    title="Add Quantity {productTitle}"
                    class="btn-icon btn-filled-primary !w-6 rounded-full !p-0"
                    on:click={() => setQuantity(++quantity)}>
                    <Add size={24} />
                </button>
            </div>
        {:else}
            <button
                title="Show Controls {productTitle}"
                class="btn-icon btn-filled-primary !w-6 rounded-full !p-0"
                on:click={setExpanded}
                in:fly>
                {quantity}
            </button>
        {/if}
    {:else if normalButton}
        <button
            title="Add Quantity {productTitle}"
            class="btn btn-sm btn-filled-primary"
            in:fly
            on:click={() => setQuantity(++quantity)}>
            Add to cart
        </button>
    {:else}
        <button
            title="Add Quantity {productTitle}"
            class="btn-icon btn-filled-primary !w-6 rounded-full !p-0"
            in:fly
            on:click={() => setQuantity(++quantity)}>
            <Add size={24} />
        </button>
    {/if}
</div>
