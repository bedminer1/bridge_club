<script lang="ts">
    import '../app.css';
    import { ModeWatcher } from "mode-watcher";
    import { page } from '$app/state';
    import { dev } from '$app/environment';
    import { injectAnalytics } from '@vercel/analytics/sveltekit'
    import SiteHeader from '$lib/components/site-header.svelte';

    let { children } = $props();
    injectAnalytics({ mode: dev ? 'development' : 'production' })
</script>

<ModeWatcher defaultMode="dark" />
<SiteHeader />
<div class="pt-14">
    {@render children()}
</div>
