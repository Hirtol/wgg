<script lang="ts">
    import { Alert, Img } from 'flowbite-svelte';
    import { AppBar, LightSwitch } from '@brainandbones/skeleton';
</script>

<LightSwitch />

<AppBar>
    <svelte:fragment slot="lead">
        <span class="text-base font-bold uppercase md:text-3xl">Logo</span>
    </svelte:fragment>
    <div class="flex flex-row">
        <input type="search" placeholder="Search..." class="mr-5 max-w-[200px] transition-all focus-within:max-w-lg" />
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
