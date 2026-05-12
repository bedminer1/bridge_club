<script lang="ts">
    import type { Snippet } from "svelte";
    import { mode } from "mode-watcher";
    let { children, index }: { children: Snippet, index: number } = $props()

    const defaultBrightness = $derived(mode.current === "dark" ? 95 : 105)
    const hoverBrightness = $derived(mode.current === "dark" ? 120 : 130)
    const shadowIntensity = $derived(mode.current === "dark" ? 15 : 30)

</script>

<div class="relative ml-[-1rem] z-{index} transition-transform brightness-{defaultBrightness} hover:brightness-{hoverBrightness} hover:shadow-amber-500 hover:shadow-xl/{shadowIntensity} hover:z-40 hover:-translate-y-1 active:brightness-125 active:shadow-amber-300 active:shadow-xl/15 active:z-40 active:-translate-y-1">
    {@render children()}
</div>

<span class="brightness-95 hover:brightness-120 hover:shadow-xl/15"></span>
<span class="brightness-105 hover:brightness-130 hover:shadow-xl/30"></span>