<!-- 
    @component
    A skeleton which takes up as much space as it can

    Can optionally `pulse` to indicate loading.
 -->
<script lang="ts">
    import classNames from 'classnames';

    export { className as class };

    export let pulse: boolean = false;

    let className: string = '';

    $: classes = classNames(
        className,
        'card card-body !bg-surface-50 dark:!bg-surface-700/75 h-full flex flex-col w-full',
        pulse && 'animate-pulse'
    );
</script>

<div class={classes}>
    <slot />
</div>
