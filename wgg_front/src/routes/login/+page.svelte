<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/stores';
    import type { ViewerContextFragment } from '$lib/api/graphql_types';
    import { notifications } from '$lib/components/notifications/notification';
    import PageRoot from '$lib/components/PageRoot.svelte';
    import { isUserAuthenticated } from '$lib/state';
    import LoginForm from './LoginForm.svelte';

    async function loginSuccess(loginResponse: CustomEvent<ViewerContextFragment>) {
        await goto($page.url.searchParams.get('redirect') ?? '/');

        notifications.info(`Signed in as: ${loginResponse.detail.username}`, 'Successful Login', 4000);
    }

    // In case we went to the login page when we're already logged in.
    if ($isUserAuthenticated) {
        goto($page.url.searchParams.get('redirect') ?? '/');
    }
</script>

<PageRoot class="flex flex-auto flex-col items-center justify-center">
    <LoginForm on:loginSuccess={loginSuccess} />
</PageRoot>
