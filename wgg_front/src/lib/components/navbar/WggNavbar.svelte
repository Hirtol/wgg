<script lang="ts">
    import { page } from '$app/stores';
    import { Navbar, NavBrand, NavLi, NavUl, NavHamburger } from 'flowbite-svelte'
    import logo from '$lib/assets/logo.svg';
    import {Logout} from 'carbon-icons-svelte';
    import { isMobileScreen } from '$lib/utils';
    import { getContextClient } from '@urql/svelte';
    import { LogoutMutationDocument } from '$lib/api/graphql_types';
    import { asyncMutationStore } from '$lib/api/urql';
    import { authSession } from '$lib/user';
    import { goto } from '$app/navigation';

    function classes(isActive: boolean): string {
      return isActive ? "!bg-primary-200 dark:!bg-primary-600 md:!bg-transparent unstyled" : "unstyled";
    }

    function linkProps(route: string, currentPage: string) {
      let isActive = currentPage == route;
      return {
        href: route,
        active: isActive,
        class: classes(isActive)
      }
    }

    async function logout() {
        const client = getContextClient();
        let _ = await asyncMutationStore({query: LogoutMutationDocument, client});
        
        authSession.set(undefined);

        await goto('/login');
    }

    $: currentPage = $page.url.pathname;    
</script>

<Navbar let:hidden let:toggle class="!bg-surface-200 dark:!bg-surface-900 sticky shadow-sm rounded">
    <NavBrand href="/" class="unstyled">
      <img
        src={logo}
        class="h-9 sm:h-12"
        alt="Wgg Logo"
      />
      <span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">
        WGG
      </span>
    </NavBrand>
    <NavHamburger on:click={toggle} />
    <NavUl {hidden} class="!bg-surface-200 dark:!bg-surface-900">
      <NavLi {...linkProps('/', currentPage)}>Home</NavLi>
      <NavLi {...linkProps('/products', currentPage)}>Products</NavLi>
      <NavLi {...linkProps('/sales', currentPage)}>Sales</NavLi>
      <NavLi {...linkProps('/cart', currentPage)}>Cart</NavLi>
      <a class="h-5 md:justify-center items-center inline-flex unstyled pl-3 md:pl-0 hover:dark:!bg-surface-700" aria-label="Logout" href="/login"
      on:click|preventDefault={logout}>
        <Logout title="Logout" class="text-primary-900 dark:text-primary-200 hover:dark:text-primary-50"></Logout>
        {#if $isMobileScreen}
           <p class="inline-block">Logout</p>
        {/if}
      </a>
    </NavUl>
</Navbar>