<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
	import { dev } from '$app/environment';
	import { injectAnalytics } from '@vercel/analytics/sveltekit'
	
	let { children } = $props();
	injectAnalytics({ mode: dev ? 'development' : 'production' })
</script>

<ModeWatcher />
{@render children()}
