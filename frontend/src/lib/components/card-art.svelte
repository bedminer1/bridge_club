<script lang="ts">
    import type { Card } from "$lib/game/types";
    import { Crown } from "@lucide/svelte";

    let { card, isIllegal, minify }: { card: Card, isIllegal: boolean | undefined, minify: boolean | undefined } = $props();

    function getAssetFileName(card: Card): string {
        const suitName = card.Suit?.toLowerCase() ?? "";
        const suitToken = suitName.includes("heart")
            ? "HEART"
            : suitName.includes("diamond")
                ? "DIAMOND"
                : suitName.includes("club")
                    ? "CLUB"
                    : suitName.includes("spade")
                        ? "SPADE"
                        : card.Suit?.toUpperCase() ?? "CLUB";

        const value = Number(card.Value ?? 0);
        const rankValue = Number.isFinite(value) && value > 0 ? value : 0;
        const normalizedValue = rankValue === 14 ? 1 : rankValue;
        const rankSuffix = normalizedValue === 1
            ? "1"
            : normalizedValue === 11
                ? "11-JACK"
                : normalizedValue === 12
                    ? "12-QUEEN"
                    : normalizedValue === 13
                        ? "13-KING"
                        : String(normalizedValue);

        return `${suitToken}-${rankSuffix}.svg`;
    }

    const assetPath = $derived(`/cards/${getAssetFileName(card)}`);
</script>

<div class="relative {minify ? 'w-[35px] sm:w-[40px]' : 'w-[43px] sm:w-[50px]'} shrink-0" style="aspect-ratio: 2.5 / 3.5;">
    {#if card.WonSet}
        <Crown class="absolute -top-6 left-0 text-accent w-5 pl-1" />
    {/if}
    <img
        src={assetPath}
        alt={`${card.Rank} of ${card.Suit}`}
        class="block w-full h-full rounded-sm border object-contain bg-white shadow-sm {isIllegal
            ? 'border-muted cursor-not-allowed brightness-75'
            : 'border-border cursor-grab'}"
    />
</div>
