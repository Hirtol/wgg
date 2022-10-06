import { Readable, readable, Subscriber } from 'svelte/store';

export const isMobileScreen: Readable<boolean> = readable(false, (set: Subscriber<boolean>) => {
    // Initialise the value
    onResize();
    window.addEventListener('resize', onResize);

    function onResize() {
        set(window.matchMedia('only screen and (max-width: 760px)').matches);
    }

    return () => {
        window.removeEventListener('resize', onResize);
    };
});