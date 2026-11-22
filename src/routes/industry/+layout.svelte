<script lang="ts">
    import { onMount } from 'svelte';
    import type { Snippet } from 'svelte';
    import { loadIndustrySettings } from '$lib/stores/industrySettings';
    import {syncSde} from "$lib/stores/sde";

    let { children }: { children: Snippet } = $props();

    onMount(() => {
        loadIndustrySettings();
        syncSde();
    });
</script>

{@render children()}