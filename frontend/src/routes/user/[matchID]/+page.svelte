<script lang="ts">
    import { page } from "$app/state";
    import ScoreDisplay from "../ScoreDisplay.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { formatDate, suitToSymbol } from "$lib/utils";
    import { Crown } from "@lucide/svelte"

    let { data } = $props()
    let { matchRecord } = $state(data)

    const playerHands: Card[][] = []
    for (let i = 1; i <= 4; i++) {
        const cardsStr = matchRecord[`player${i}Hand` as keyof MatchRecord] as string
        playerHands.push(JSON.parse(cardsStr))
    }

    /** Parse completed sets data from DB (JSON string) */
    interface SavedCompletedSet {
        Cards: Card[]
        WinnerID: number
        PlayerIDs: number[]
    }
    let completedSets: SavedCompletedSet[] = $derived.by(() => {
        if (!matchRecord.setsData) return []
        try {
            return JSON.parse(matchRecord.setsData)
        } catch {
            return []
        }
    })

    const playerColor: Record<number, string> = {
        1: 'var(--red)',
        2: 'var(--blue)',
        3: 'var(--yellow)',
        4: 'var(--green)',
    }
    const playerShort: Record<number, string> = {
        1: 'P1',
        2: 'P2',
        3: 'P3',
        4: 'P4',
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

        <!-- Completed sets (set-by-set replay) -->
        {#if completedSets.length > 0}
        <div class="rounded-lg border border-border bg-card/40 p-4 text-sm">
            <h3 class="text-sm font-semibold text-muted-foreground mb-3">Sets Played</h3>
            <div class="flex flex-col gap-3">
                {#each completedSets as set, setIdx}
                <div class="flex items-start gap-2 rounded bg-muted/30 p-2">
                    <span class="text-xs text-muted-foreground w-6 shrink-0 mt-2">#{setIdx + 1}</span>
                    <div class="flex gap-2 items-start">
                        {#each set.Cards as card, ci}
                        {@const pid = set.PlayerIDs?.[ci] ?? 0}
                        <div class="flex flex-col items-center gap-0.5 min-w-[35px]">
                            <PokerCard card={card} isIllegal={false} minify={true} />
                            <span class="text-[10px] font-medium leading-none" style="color: {playerColor[pid] ?? 'var(--muted-foreground)'}">
                                {playerShort[pid] ?? ''}
                            </span>
                        </div>
                        {/each}
                    </div>
                    <div class="flex items-center gap-1 ml-1 shrink-0 mt-1.5">
                        <Crown class="w-3.5 h-3.5 text-accent" />
                        <span class="text-xs font-semibold" style="color: {playerColor[set.WinnerID] ?? 'var(--foreground)'}">
                            {playerShort[set.WinnerID] ?? 'P' + set.WinnerID}
                        </span>
                    </div>
                </div>
                {/each}
            </div>
        </div>
        {/if}

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
