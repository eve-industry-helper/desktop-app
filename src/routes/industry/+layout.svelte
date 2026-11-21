<script lang="ts">
    import { onMount } from 'svelte';
    import { moduleSubNav } from '$lib/stores/global';
    import { loadIndustrySettings } from './settings/settings';

    $: moduleSubNav.set([
        { label: "Plans", href: "/industry/plans" },
        { label: "Settings", href: "/industry/settings" }
    ]);

    onMount(() => {
        loadIndustrySettings();
    });
</script>

<slot />