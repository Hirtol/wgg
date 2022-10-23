<script lang="ts">
    import classNames from 'classnames';

    export {className as class};

    export let title: string | undefined = undefined;

    export let blurImage: boolean = false;

    export let data: { imageUrl?: string; name: string };

    let className: string;

    $: classes = classNames(
        'aspect-video h-[15vh] cursor-pointer object-contain hover:object-scale-down md:h-[12vh]',
        className
    );
</script>

<img
    src={data.imageUrl}
    draggable="false"
    loading="lazy"
    class={classes}
    class:opacity-20={blurImage}
    {title}
    alt={data.name} />
