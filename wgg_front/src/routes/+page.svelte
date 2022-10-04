<script lang="ts">
    import { Alert, Img } from 'flowbite-svelte';
    import { AppBar, LightSwitch } from '@brainandbones/skeleton';
    import { FragmentType, graphql, useFragment } from '$lib/api/graphql_types';
    import { AutoCompleteFragmentFragment } from '$lib/api/graphql_types/graphql';
    import { PageData } from '$types';
    
    const testo = graphql(/* GraphQL */ `
        fragment AutoCompleteFragment on WggAutocomplete {
            name
        }
    `);

    export let data: PageData;
    $: ({result} = data);
</script>

<LightSwitch />

<div class="overflow-auto">
    {#if $result.data}
        {#each $result.data.proSearchAll as item}
        <Img src={item.imageUrl} size="max-w-[10rem]" class=""></Img>
        {/each}
    {/if}
</div>


<AppBar>
    <svelte:fragment slot="lead">
        <span class="text-base md:text-3xl font-bold uppercase">Logo</span>
    </svelte:fragment>
    <div class="flex flex-row">
        <input type="search" placeholder="Search..." class="max-w-[200px] focus-within:max-w-lg transition-all mr-5" />
    </div>
    <svelte:fragment slot="trail">
        <div class="flex items-center space-x-6">
            <a href="/components/app-bar">Link</a>
        </div>
    </svelte:fragment>
</AppBar>

<h1 class="mx-auto">Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
<div class="mx-auto">
    <Alert>
        <span class="font-medium">Info Alert!</span> Change stuff
    </Alert>
</div>
