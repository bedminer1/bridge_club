<script lang="ts">
    import type { Snippet } from "svelte";
    let { children, index }: { children: Snippet, index: number } = $props()
</script>

<!-- Hand card with hover lift + accent shadow -->
<div
    class="relative ml-[-1rem] z-[{index}] transition-all hover:z-40 hover:-translate-y-2 hover:shadow-accent hover:shadow-xl/30 active:-translate-y-1 active:shadow-accent active:shadow-lg/20"
>
    {@render children()}
</div>
