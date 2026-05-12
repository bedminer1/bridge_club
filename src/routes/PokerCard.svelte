<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js";
    import { suitToColor, suitToSymbol } from "$lib/utils";
    import { Crown } from "@lucide/svelte"
    let { card, isIllegal }: { card: Card, isIllegal: boolean } = $props()


    const symbol = $derived(suitToSymbol.get(card.Suit))
    const color = $derived(suitToColor.get(card.Suit))
</script>

<div class="relative w-[50px] h-[60px]">
    {#if card.WonSet}
        <Crown class="absolute -top-6 left-0 text-amber-200 w-5 pl-1" />
    {/if}
    <Card.Root class="w-full h-full rounded-sm p-0 {isIllegal ? "bg-zinc-50 cursor-not-allowed brightness-70" : "bg-zinc-50 cursor-grab"}">
    <Card.Content class={`p-0 text-lg text-${color}`}>
        <p>{card.Rank} {symbol}</p>
    </Card.Content>
    </Card.Root>
</div>

<!-- for safelist -->
<span class="text-red-500"></span>
<span class="text-black"></span>