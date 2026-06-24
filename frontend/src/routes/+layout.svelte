<script lang="ts">
    import '../app.css';
    import { ModeWatcher } from "mode-watcher";
    import SiteHeader from '$lib/components/site-header.svelte';
    import { headerState } from "$lib/game/header-state.svelte";

    let { children } = $props();
</script>

<ModeWatcher defaultMode="dark" />
{#if headerState.showSiteHeader}
    <SiteHeader />
{/if}
<div class={headerState.showSiteHeader ? "pt-14" : ""}>
    {@render children()}
</div>
