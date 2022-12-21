<script lang="ts">
    import { page } from '$app/stores';
    console.log($page.error?.message);
    
</script>

<main class="container mx-auto">
    <h1>The requested product/provider combination didn't exist!</h1>
    <p>{$page.status}: {$page.error?.message}</p>
</main>
