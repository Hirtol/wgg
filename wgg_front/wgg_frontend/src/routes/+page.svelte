<script>
    import { Alert } from 'flowbite-svelte';
    import { AppBar, LightSwitch } from '@brainandbones/skeleton';

    function t() {
        if (true) {
            return '';
        }
    }
</script>

<LightSwitch />

<AppBar>
    <svelte:fragment slot="lead">
        <span class="text-base md:text-3xl font-bold uppercase">Logo</span>
    </svelte:fragment>
    <input type="search" placeholder="Search..." class="max-w-[200px]" />
    <svelte:fragment slot="trail">
        <div class="flex items-center space-x-6">
            <a href="/components/app-bar">Link</a>
        </div>
    </svelte:fragment>
</AppBar>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
<div class="justify-center flex">
    <Alert>
        <span class="font-medium">Info Alert!</span> Change stuff
    </Alert>
</div>
