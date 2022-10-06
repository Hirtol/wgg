<script lang="ts">
    import classNames from 'classnames';
    import { globalLoading, Progress } from './global_loading';

    const fillPercentages: { [K in Progress]: number } = { Begin: 0, Starting: 25, Loading: 50, Idle: 100 };

    let transitionComplete = true;

    $: currentProgress = $globalLoading;
    $: loading = currentProgress != Progress.Idle;
    $: classes = classNames(
        $$props.class,
        transitionComplete ? 'invisible' : 'visible',
        'fixed top-0 progress-wrapper w-full h-1'
    );
    $: fillPercent = transitionComplete ? 0 : fillPercentages[currentProgress];

    $: if (loading) transitionComplete = false;
</script>

<div
    class={classes}
    data-testid="progress-wrapper"
    role="progressbar"
    aria-label={!transitionComplete ? 'Loading' : undefined}
    aria-hidden={transitionComplete}>
    <div
        on:transitionend={() => (transitionComplete = !loading)}
        class="progress-meter h-full animate-pulse rounded-xl bg-accent-500 transition-[width] duration-500 ease-in-out"
        style:width="{fillPercent}%" />
</div>
