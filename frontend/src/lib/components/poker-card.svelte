<script lang="ts">
    import { suitToSymbol } from "$lib/utils";
    import { Crown } from "@lucide/svelte"

    let { card, isIllegal, minify }: { card: Card, isIllegal: boolean | undefined, minify: boolean | undefined } = $props()

    const symbol = $derived(suitToSymbol.get(card.Suit))

    /** Card suit text color — use explicit hex values to avoid Tailwind JIT issues */
    const cardColor = $derived(
        card.Suit === "Heart" || card.Suit === "Diamond" ? "#dc2626" : "#111827"
    )
</script>

<div class="relative {minify ? "w-[35px] h-[40px] text-[9px]" : "w-[43px] h-[52px] text-sm sm:w-[50px] sm:h-[60px] sm:text-base"}" style="flex-shrink: 0;">
    {#if card.WonSet}
        <Crown class="absolute -top-6 left-0 text-accent w-5 pl-1" />
    {/if}
    <div
        class="w-full h-full rounded-sm border bg-white p-0.5 {isIllegal
            ? 'border-muted cursor-not-allowed brightness-75'
            : 'border-border cursor-grab'}"
    >
        <!-- Rank + suit in top-right corner -->
        <div class="flex flex-col items-end leading-none" style="color: {cardColor}">
            <span class="font-bold {isIllegal ? 'opacity-70' : ''}">{card.Rank}</span>
            <span class="{isIllegal ? 'opacity-70' : ''}">{symbol}</span>
        </div>
    </div>
</div>
