<script lang="ts">
    import '../app.css';
    import { ModeWatcher } from "mode-watcher";
    import SiteHeader from '$lib/components/site-header.svelte';

    let { children } = $props();
</script>

<ModeWatcher defaultMode="dark" />
<SiteHeader />
<div class="pt-14">
    {@render children()}
</div>
