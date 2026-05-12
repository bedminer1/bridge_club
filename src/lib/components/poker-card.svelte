<script lang="ts">
    import { suitToColor, suitToSymbol } from "$lib/utils";
    import { Crown } from "@lucide/svelte"

    let { card, isIllegal, minify }: { card: Card, isIllegal: boolean | undefined, minify: boolean | undefined } = $props()

    const symbol = $derived(suitToSymbol.get(card.Suit))
    const color = $derived(suitToColor.get(card.Suit))
</script>

<div class="relative {minify ? "w-[35px] h-[40px] text-sm" : "w-[43px] h-[52px] text-sm sm:w-[50px] sm:h-[60px] sm:text-lg"}">
    {#if card.WonSet}
        <Crown class="absolute -top-6 left-0 text-accent w-5 pl-1" />
    {/if}
    <div
        class="w-full h-full rounded-sm border bg-white p-1 {isIllegal
            ? "border-muted cursor-not-allowed"
            : "border-border cursor-grab"}"
    >
        <p class="text-{color} {isIllegal ? 'opacity-60' : ''} leading-none">{card.Rank}<br>{symbol}</p>
    </div>
</div>
