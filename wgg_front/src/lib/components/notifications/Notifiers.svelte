<script lang="ts">
    import { notifications, NotificationType } from '$lib/components/notifications/notification';
    import { CloseButton } from 'flowbite-svelte';

    import { flip } from 'svelte/animate';
    import { fly } from 'svelte/transition';
    import Toast from './Toast.svelte';
    import { Checkmark, Close, Information, Warning } from 'carbon-icons-svelte';
    import type { SvelteComponent } from 'svelte';
    
    const icons: {[K in NotificationType]: typeof SvelteComponent} = {'success': Checkmark, 
'error': Close, 'info': Information, warning: Warning};

    /**
     * The three most recent notifications are the only ones displayed to avoid spamming the view.
     */
    $: mostRecent = $notifications.slice(0, 2);
</script>

<ul class="fixed bottom-8 left-0 md:left-auto right-0 flex flex-col justify-start items-center z-50 list-none md:mr-4">
    {#each mostRecent as notification, i (notification.id)}
        <li animate:flip>
            <div class="py-0.5" transition:fly={{ x: 100, duration: 500 }}>
                <Toast iconBgColor={notification.bgColorClass} color={notification.color} on:click={() => notifications.remove(i)} class="bg-surface-800">
                    <svelte:fragment slot="icon">
                            <svelte:component this={icons[notification.type]}></svelte:component>
                    </svelte:fragment>
                    {notification.message }
                </Toast>
            </div>
        </li>
    {/each}
</ul>
