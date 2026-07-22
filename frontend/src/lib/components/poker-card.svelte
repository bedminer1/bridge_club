<script lang="ts">
    import CardArt from "$lib/components/card-art.svelte";
    import type { Card } from "$lib/game/types";

    let { card, isIllegal, minify }: { card: Card, isIllegal: boolean | undefined, minify: boolean | undefined } = $props();
</script>

<CardArt card={card} isIllegal={isIllegal} minify={minify} />
