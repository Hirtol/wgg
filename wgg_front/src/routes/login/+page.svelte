<script lang="ts">
    import { ViewerContextFragment } from '$lib/api/graphql_types';
    import { authSession } from '$lib/user';
    import type { PageData } from './$types';
    import LoginForm from './LoginForm.svelte';

    export let data: PageData;
</script>

<main class="content m-auto">
    <LoginForm on:loginSuccess={(e) => authSession.set(e.detail)} />
</main>
