<script lang="ts">
    import classNames from 'classnames';

    export { _class as class };

    export let title: string | undefined = undefined;

    export let blurImage: boolean = false;

    export let data: { imageUrl?: string; name: string };

    let _class: string = '';

    $: classes = classNames(
        'aspect-video h-24 cursor-pointer object-contain hover:object-scale-down md:h-32',
        _class
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
