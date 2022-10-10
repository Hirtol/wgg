<script lang="ts">
    import { goto } from '$app/navigation';
    import { getContextClient } from '$lib/api/urql';
    import WggNavbar from '$lib/components/navbar/WggNavbar.svelte';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { authSession, logoutUser } from '$lib/user';

    const client = getContextClient();

    async function logout() {
        await logoutUser(client);

        await goto('/login');
    }
</script>

<PageRoot>
    {#if $authSession}
        <WggNavbar user={$authSession} on:logout={logout} />
    {/if}

    <slot />
</PageRoot>
