<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
	import { page } from '$app/state';
	import { dev } from '$app/environment';
	import { injectAnalytics } from '@vercel/analytics/sveltekit'
	import { Banana, History } from '@lucide/svelte';
	
	let { children } = $props();
	injectAnalytics({ mode: dev ? 'development' : 'production' })
</script>

<ModeWatcher />
<div class="absolute top-2 left-2">
	<div class="flex gap-3">
		<a href="/" class={page.url.pathname === "/" ? "opacity-100" : "opacity-55"}><Banana /></a>
		<a href="/history" class={page.url.pathname === "/history" ? "opacity-100" : "opacity-55"}><History /></a>
	</div>
</div>
{@render children()}
