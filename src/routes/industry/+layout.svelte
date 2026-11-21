<script lang="ts">
    import { onMount } from 'svelte';
    import type { Snippet } from 'svelte';
    import { loadIndustrySettings } from '$lib/stores/industrySettings';

    let { children }: { children: Snippet } = $props();

    onMount(() => {
        loadIndustrySettings();
    });
</script>

{@render children()}