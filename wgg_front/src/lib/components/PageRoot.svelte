<script lang="ts">
    import classNames from "classnames";

    export {_class as class};

    let _class: string = '';
</script>

<main class={classNames("container mx-auto px-0.5 md:px-0", _class)}>
    <slot/>
</main>
