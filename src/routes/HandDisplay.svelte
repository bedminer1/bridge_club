<script lang="ts">
    import type { Snippet } from "svelte";
    let { children, index }: { children: Snippet, index: number } = $props()
</script>

<div class="relative ml-[-1rem] z-{index} transition-transform brightness-95 hover:brightness-125 hover:shadow-amber-300 hover:shadow-xl/15 hover:z-40 hover:-translate-y-1">
    {@render children()}
</div>