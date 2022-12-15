<!-- 
    @component
    A skeleton which takes up as much space as it can

    Can optionally `pulse` to indicate loading.
 -->
<script lang="ts">
    import classNames from 'classnames';

    export { _class as class };

    export let pulse: boolean = false;

    let _class: string = '';

    $: classes = classNames(
        'card p-4 !bg-surface-50 dark:!bg-surface-700/75 h-full flex flex-col w-full',
        pulse && 'animate-pulse',
        _class
    );
</script>

<div class={classes}>
    <slot />
</div>
