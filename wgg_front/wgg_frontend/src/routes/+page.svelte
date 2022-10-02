<script>
    import { Alert } from 'flowbite-svelte';
    import { AppBar, LightSwitch } from '@brainandbones/skeleton';
</script>

<LightSwitch />

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
