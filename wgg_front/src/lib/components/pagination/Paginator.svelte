<script context="module" lang="ts">
    export interface PaginationSettings {
        offset: number;
        limit: number;
    }
</script>

<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    const dispatch = createEventDispatcher();
    // Props
    /**
     * Pass the page setting object.
     * @type {PaginationSettings}
     */
    export let settings: PaginationSettings = { offset: 0, limit: 5 };
    export let totalCount: number;
    export let amounts: number[] = [5, 10, 20];
    // Props (design)
    /** Provide classes to set flexbox justification. */
    export let justify = 'justify-between';
    /** Provide classes to style page info text. */
    export let text = 'text-xs';
    /** Provide arbitrary classes to style the select input. */
    export let select: string | undefined = undefined;
    /** Sets selection and buttons to disabled state on-demand. */
    export let disabled = false;
    // Props (Select & Buttons)
    /** Set the text for the amount selection input. */
    export let amountText = 'Items';
    /** Provide abtitrary classes to the next/previous buttons. */
    export let buttonClasses = 'btn-icon btn-filled';
    /** Set the text label for the Previous button. */
    export let buttonTextPrevious = '&larr;';
    /** Set the text label for the Next button. */
    export let buttonTextNext = '&rarr;';
    // Base Classes
    const cBase = 'flex flex-col md:flex-row items-center space-y-4 md:space-y-0 md:space-x-4';
    const cLabel = 'w-full md:w-auto';
    const cPageText = 'whitespace-nowrap';

    // Functionality
    function onChangeLength(): void {
        settings.offset = 0;
        /** @event {{ length: number }} amount - Fires when the amount selection input changes.  */
        dispatch('amount', settings.limit);
    }
    function onPrev(): void {
        settings.offset--;
        /** @event {{ offset: number }} page Fires when the next/back buttons are pressed. */
        dispatch('page', settings.offset);
    }
    function onNext(): void {
        settings.offset++;
        dispatch('page', settings.offset);
    }

    // Reactive Classes
    $: classesBase = `${cBase} ${justify} ${$$props.class ?? ''}`;
    $: classesLabel = `${cLabel}`;
    $: classesSelect = `${select}`;
    $: classesPageText = `${cPageText} ${text}`;
</script>

<div class="paginator {classesBase}" data-testid="paginator">
    <!-- Select Amount -->
    <label class="paginator-label {classesLabel}">
        <select
            bind:value={settings.limit}
            on:change={() => {
                onChangeLength();
            }}
            class="paginator-select {classesSelect}"
            {disabled}
            aria-label="Select Amount">
            {#each amounts as amount}<option value={amount}>{amount} {amountText}</option>{/each}
        </select>
    </label>
    <!-- Details -->
    <span class="paginator-details {classesPageText}">
        {settings.offset * settings.limit + 1} - {Math.min(
            settings.offset * settings.limit + settings.limit,
            totalCount
        )} <span class="px-2 opacity-50">/</span> <strong>{totalCount}</strong>
    </span>
    <!-- Arrows -->
    <div class="paginator-arrows space-x-2">
        <button
            class={buttonClasses}
            on:click={() => {
                onPrev();
            }}
            disabled={disabled || settings.offset === 0}>
            {@html buttonTextPrevious}
        </button>
        <button
            class={buttonClasses}
            on:click={() => {
                onNext();
            }}
            disabled={disabled || (settings.offset + 1) * settings.limit >= totalCount}>
            {@html buttonTextNext}
        </button>
    </div>
</div>
