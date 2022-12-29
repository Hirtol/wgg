<!--
    @component
    Very simple table to display some data with selectable properties
-->
<script context="module" lang="ts">
    export interface TableData {
        /** The formatted table heading values. */
        head: string[];
        /** The data returned when an interactive row is clicked. */
        meta?: string[][];
        /** The formatted table footer values. */
        foot?: string[];
    }

    export interface TableRow {
        index: string;
        checked: boolean;
        data: any;
    }
</script>

<script lang="ts">
    import { createEventDispatcher } from 'svelte';

    export { _class as class };

    export let settings: TableData;
    /** The formatted table body values. */
    export let data: TableRow[];
    export let withSelectAll: boolean = true;

    // Props (styles)
    /** Override the Tailwind Element class. Replace this for a headless UI. */
    export let tableClass = 'table table-hover';
    /** Provide arbitrary classes for the table head. */
    export let regionHead = '';
    /** Provide arbitrary classes for the table body. */
    export let regionBody = '';
    /**
     * Classes for the individual body rows.
     */
    export let rowClass = '';
    /** Provide arbitrary classes for the table foot. */
    export let regionFoot = '';

    const dispatcher = createEventDispatcher<{
        allSelected: boolean;
        selected: TableRow;
    }>();

    let _class: string = '';
</script>

<div class="table-container {_class}">
    <table class={tableClass}>
        <!-- Head -->
        <thead class={regionHead}>
            <tr>
                <th>
                    {#if withSelectAll}
                        <input type="checkbox" on:change={(x) => dispatcher('allSelected', x.currentTarget.checked)} />
                    {/if}
                    <!-- Selection -->
                </th>

                {#each settings.head as item}
                    <th>{item}</th>
                {/each}
            </tr>
        </thead>
        <!-- Body -->
        <tbody class={regionBody}>
            {#each data as item, i}
                <tr class={rowClass} class:table-row-checked={item.checked}>
                    <td role="gridcell">
                        <input
                            type="checkbox"
                            bind:checked={item.checked}
                            on:change={() => dispatcher('selected', item)} />
                    </td>
                    <slot {item} />
                </tr>
            {/each}
        </tbody>
        <!-- Foot -->
        {#if settings.foot}
            <tfoot class="table-foot {regionFoot}">
                <tr>
                    {#each settings.foot as cell}
                        <td>{@html cell}</td>
                    {/each}
                </tr>
            </tfoot>
        {/if}
    </table>
</div>
