<script lang="ts">
    import classNames from "classnames";

    export {clasz as class};

    let clasz: string = ''
</script>

<main class={classNames("container mx-auto px-0.5 md:px-0", clasz)}>
    <slot/>
</main>
