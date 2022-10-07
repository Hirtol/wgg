<script lang="ts">
    import { SubmitLoginDocument, type ViewerContextFragment } from '$lib/api/graphql_types';
    import { asyncMutationStore, getContextClient } from '$lib/api/urql';
    import { globalLoading } from '$lib/components/global_progress/global_loading';
    import { Email, Password } from 'carbon-icons-svelte';
    import { Button, Card, Checkbox, Helper, Input, Label } from 'flowbite-svelte';
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher<{
        loginSuccess: ViewerContextFragment;
        loginFailed: any;
    }>();

    let email: string;
    let password: string;

    let awaitingLogin: boolean;
    let failedLogin: boolean;
    const client = getContextClient();

    $: buttonDisabled = !(email && password) || awaitingLogin;

    async function submitLogin(e: SubmitEvent) {
        awaitingLogin = true;

        try {
            const { item } = await globalLoading.submit(
                asyncMutationStore({
                    query: SubmitLoginDocument,
                    variables: { email, password },
                    client
                })
            );

            dispatch('loginSuccess', item.login.user);
        } catch (error) {
            failedLogin = true;
            dispatch('loginFailed', error);
        }

        awaitingLogin = false;
    }
</script>

<main class="h-full w-full">
    <Card {...$$restProps} class="!bg-surface-900" size="lg">
        <form on:submit|preventDefault={submitLogin} class="flex flex-col space-y-6 md:w-[30vw] ">
            <h2 class="p-0 text-xl font-medium text-gray-900 dark:text-white">Please sign in</h2>
            <Label class="space-y-2">
                <span>Email</span>
                <Input
                    type="email"
                    name="email"
                    placeholder="name@email.com"
                    required
                    bind:value={email}
                    class={failedLogin ? 'bg-warning-400 dark:!bg-warning-700' : ''}>
                    <Email slot="left" aria-hidden="true" />
                </Input>
                {#if failedLogin}
                    <Helper class="mt-2" color="red">Either the email or password was incorrect</Helper>
                {/if}
            </Label>
            <Label class="space-y-2">
                <span>Your password</span>
                <Input
                    type="password"
                    name="password"
                    placeholder="•••••"
                    required
                    bind:value={password}
                    class={failedLogin ? 'bg-warning-400 dark:!bg-warning-700' : ''}>
                    <Password slot="left" aria-hidden="true" />
                </Input>
                {#if failedLogin}
                    <Helper class="mt-2" color="red">Either the email or password was incorrect</Helper>
                {/if}
            </Label>
            <Button
                type="submit"
                class="w-full"
                disabled={buttonDisabled}
                title={buttonDisabled ? 'Please fill in both fields above' : 'Log in'}>
                Login
            </Button>
        </form>
    </Card>
</main>
