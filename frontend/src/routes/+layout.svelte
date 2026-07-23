<script lang="ts">
    import '../app.css';
    import { onMount } from "svelte";
    import { ModeWatcher } from "mode-watcher";
    import SiteHeader from '$lib/components/site-header.svelte';
    import { preloadCardAssets } from "$lib/game/card-asset-preload";
    import { headerState } from "$lib/game/header-state.svelte";

    let { children } = $props();

    onMount(() => {
        preloadCardAssets();
    });
</script>

<ModeWatcher defaultMode="dark" />
{#if headerState.showSiteHeader}
    <SiteHeader />
{/if}
<div class={headerState.showSiteHeader ? "pt-14" : ""}>
    {@render children()}
</div>
