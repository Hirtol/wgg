<script lang="ts">
    export { _class as class };

    let _class: string = '';
</script>

<main class="container mx-auto px-0.5 pt-4 md:px-0 {_class}">
    <slot />
</main>
