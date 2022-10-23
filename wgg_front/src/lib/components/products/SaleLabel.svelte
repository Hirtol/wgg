<script lang="ts">
    import classNames from 'classnames';

    export { className as class };
    export let text: string;

    let className: string | undefined = undefined;
</script>

<span class={classNames('badge w-min bg-primary-300 dark:bg-primary-800', className)}>{text}</span>
