<script lang="ts">
    import WggNavbar from '$lib/components/navbar/WggNavbar.svelte';
    import PageRoot from '$lib/components/PageRoot.svelte';
</script>

<PageRoot>
    <WggNavbar />
    
    <slot />
</PageRoot>
