<script lang="ts">
    import { page } from '$app/stores';
    import logo from '$lib/assets/logo.svg';
    import { aggregatePageRootUrl, cartPageRootUrl, productPageRootUrl, salesPageRootUrl } from '$lib/routing';
    import { UserData } from '$lib/state';
    import { isMobileScreen } from '$lib/state';
    import { Logout, Settings } from 'carbon-icons-svelte';
    import { Navbar, NavBrand, NavHamburger, NavLi, NavUl } from 'flowbite-svelte';
    import { createEventDispatcher } from 'svelte';

    export let user: UserData;

    const dispatch = createEventDispatcher<{
        logout: void;
    }>();

    function classes(isActive: boolean): string {
        return isActive ? '!bg-primary-200 dark:!bg-primary-600 md:!bg-transparent unstyled' : 'unstyled';
    }

    function linkProps(route: string, currentPage: string) {
        let isActive = (currentPage.includes(route) && route != '/') || currentPage == route;
        return {
            href: route,
            active: isActive,
            class: classes(isActive)
        };
    }

    $: currentPage = $page.url.pathname;
</script>

<Navbar let:hidden let:toggle class="sticky rounded !bg-surface-200 shadow-sm dark:!bg-surface-900">
    <NavBrand href="/" class="unstyled">
        <img src={logo} class="aspect-square h-12 w-12 sm:h-12" alt="Wgg Logo" />
        <span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white"> WGG </span>
    </NavBrand>
    <NavHamburger on:click={toggle} />
    <NavUl {hidden} class="!bg-surface-200 dark:!bg-surface-900">
        <NavLi {...linkProps('/', currentPage)}>Home</NavLi>
        <NavLi {...linkProps(productPageRootUrl, currentPage)}>Products</NavLi>
        <NavLi {...linkProps(aggregatePageRootUrl, currentPage)}>Aggregates</NavLi>
        <NavLi {...linkProps(salesPageRootUrl, currentPage)}>Sales</NavLi>
        <NavLi {...linkProps(cartPageRootUrl, currentPage)}>Cart</NavLi>
        <a
            class="unstyled inline-flex h-5 items-center py-2 pl-3  hover:dark:!bg-surface-700 md:justify-center md:pl-0"
            aria-label="Logout"
            href="/login"
            on:click|preventDefault={() => dispatch('logout')}>
            <Logout title="Logout" class="text-primary-900 dark:text-primary-200 hover:dark:text-primary-50" />
            {#if $isMobileScreen}
                <p class="inline-block">Logout</p>
            {/if}
        </a>

        <a
            class="unstyled inline-flex h-5 items-center py-2 pl-3  hover:dark:!bg-surface-700 md:justify-center md:pl-0"
            aria-label="Settings"
            href="/settings">
            <Settings title="Settings" class="text-primary-900 dark:text-primary-200 hover:dark:text-primary-50" />
        </a>

        <li>
            <a class="unstyled py-2 pl-3" href="/">{user.username}</a>
        </li>
    </NavUl>
</Navbar>
