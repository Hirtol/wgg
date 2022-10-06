<script lang="ts">
    import classNames from 'classnames';
    import { CloseButton } from 'flowbite-svelte';

    export let iconBgColor: string = 'bg-primary-800';
    export let divClass: string = 'w-full max-w-xs p-4 rounded-lg border border-gray-500';

    let classDiv: string;
    $: classDiv = classNames(divClass, $$props.class, 'w-[90vw] md:w-[18rem] h-auto');

    let iconClass: string;
    $: iconClass = classNames(
        'rounded-lg inline-flex items-center justify-center flex-shrink-0 w-8 h-8 mr-3 border border-solid',
        iconBgColor
    );
</script>

<div {...$$restProps} class={classDiv} role="alert">
    <div class="flex {$$slots.extra ? 'items-start' : 'items-center'}">
        {#if $$slots.icon}
            <div class={iconClass}><slot name="icon" /></div>
        {/if}

        <div class="w-full text-sm font-normal">
            <slot />
            <slot name="extra" />
        </div>

        <CloseButton on:click />
    </div>
</div>
