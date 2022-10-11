<script lang="ts">
    import OutroBlocker from './OutroBlocker.svelte';
</script>

<OutroBlocker>
    <main class="overflow-auto">
        <slot />
    </main>
</OutroBlocker>
