<script lang="ts">
    import { getContextClient } from '$lib/api/urql';
    import { globalLoading } from '$lib/components/global_progress/global_loading';
    import { loginUser, UserData } from '$lib/state';
    import { Email, Password } from 'carbon-icons-svelte';
    import { Button, Card, Helper, Input, Label } from 'flowbite-svelte';
    import { createEventDispatcher } from 'svelte';

    export { _class as class };

    const client = getContextClient();
    const dispatch = createEventDispatcher<{
        loginSuccess: UserData;
        loginFailed: any;
    }>();

    let _class: string = '';

    let email: string;
    let password: string;

    let awaitingLogin: boolean;
    let failedLogin: boolean;

    $: buttonDisabled = !(email && password) || awaitingLogin;

    async function submitLogin(e: SubmitEvent) {
        awaitingLogin = true;

        try {
            const { item } = await loginUser(email, password, client);

            dispatch('loginSuccess', item);
        } catch (error) {
            failedLogin = true;
            dispatch('loginFailed', error);
        }

        awaitingLogin = false;
    }
</script>

<Card class="w-full dark:!bg-surface-900 {_class}" size="md">
    <form on:submit|preventDefault={submitLogin} class="flex max-w-full flex-col space-y-6">
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
