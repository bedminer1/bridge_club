<script lang="ts">
    import { page } from "$app/state";
    import ScoreDisplay from "../ScoreDisplay.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { formatDate, suitToSymbol } from "$lib/utils";

    let { data } = $props()
    let { matchRecord } = $state(data)

    const playerHands: Card[][] = []
    for (let i = 1; i <= 4; i++) {
        const cardsStr = matchRecord[`player${i}Hand` as keyof MatchRecord] as string
        playerHands.push(JSON.parse(cardsStr))
    }
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <div class="w-full max-w-3xl flex flex-col gap-6">
        <!-- Result header -->
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span
                    class="text-lg font-bold {matchRecord.wonMatch ? 'text-[var(--green)]' : 'text-[var(--red)]'}"
                >
                    {matchRecord.wonMatch ? "Victory" : "Defeat"}
                </span>
                <span class="text-xs text-muted-foreground">|</span>
                <span class="text-xs text-muted-foreground">{formatDate(matchRecord.date)}</span>
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <span class="rounded border border-border px-1.5 py-0.5">{matchRecord.betSize}{suitToSymbol.get(matchRecord.trumpSuit)}</span>
                <span>{matchRecord.botDifficulty}</span>
            </div>
        </div>

        <!-- Score display -->
        <div class="rounded-lg w-auto border border-border bg-card/60 p-3 text-sm">
            <ScoreDisplay {matchRecord} />
        </div>

        <!-- Player hands -->
        <div class="flex flex-col gap-3 rounded-lg w-auto border border-border bg-card/40 p-3 text-sm">
            {#each playerHands as playerHand, playerID}
                {@const pid = playerID + 1}
                {@const colorVar = pid === 1 ? '--red' : pid === 2 ? '--blue' : pid === 3 ? '--yellow' : '--green'}
                {@const sets = matchRecord[`player${pid}Sets` as keyof MatchRecord] as number}
                <div class="p-3">
                    <div class="flex items-center gap-2 mb-6 text-xs">
                        <span class="font-medium" style="color: var({colorVar})">P{pid}</span>
                        <span class="text-muted-foreground">{sets} sets</span>
                    </div>
                    <div class="flex ml-5">
                        {#each playerHand as card, index}
                            <HandDisplay {index}>
                                <PokerCard card={card} isIllegal={false} minify={true} />
                            </HandDisplay>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    </div>
</div>
