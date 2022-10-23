<script lang="ts">
    import { goto } from '$app/navigation';
    import { getContextClient } from '$lib/api/urql';
    import WggNavbar from '$lib/components/navbar/WggNavbar.svelte';
    import OutroBlocker from '$lib/components/OutroBlocker.svelte';
    import { authSession, logoutUser } from '$lib/state';

    const client = getContextClient();

    async function logout() {
        await logoutUser(client);

        await goto('/login');
    }
</script>

<!-- Block outro transitions when dismounting the page -->
<OutroBlocker>
    <!-- Show scroll-bar for main app content -->
    <main class="overflow-auto">
        {#if $authSession}
            <WggNavbar user={$authSession} on:logout={logout} />
        {/if}

        <slot />
    </main>
</OutroBlocker>
