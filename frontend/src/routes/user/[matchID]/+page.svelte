<script lang="ts">
    import { page } from "$app/state";
    import ScoreDisplay from "../ScoreDisplay.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import { formatDate, suitToSymbol } from "$lib/utils";

    let { data } = $props()
    let { matchRecord, userID } = $state(data)

    /** Determine match result for the current user */
    let didWin = $derived.by(() => {
        if (!matchRecord) return false
        // New fields (multiplayer): use betWinnerUserId / partnerUserId / winningTeam
        if (matchRecord.betWinnerUserId != null && matchRecord.betWinnerUserId !== 0 && matchRecord.winningTeam != null) {
            const viewerOnTeam1 = matchRecord.betWinnerUserId === userID || matchRecord.partnerUserId === userID
            const viewerTeam = viewerOnTeam1 ? 1 : 2
            return viewerTeam === matchRecord.winningTeam
        }
        // Fallback to old single-player field
        return !!matchRecord.wonMatch
    })

    /** Parse each player's played cards from DB (JSON array of Card objects) */
    const playerPlayedCards: Card[][] = []
    if (matchRecord) {
        for (let i = 1; i <= 4; i++) {
            const cardsStr = matchRecord[`player${i}Hand` as keyof typeof matchRecord] as string
            try {
                playerPlayedCards.push(JSON.parse(cardsStr))
            } catch {
                playerPlayedCards.push([])
            }
        }
    }

    /** Parse the players field: JSON array of { id, username } */
    const playersMeta: Array<{ id: number; username: string }> = (() => {
        if (!matchRecord) return []
        try {
            return JSON.parse(matchRecord.players || "[]")
        } catch {
            return []
        }
    })()

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
        <!-- Back arrow -->
        <a href="/user" class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors w-fit">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
            Back
        </a>

        <!-- Result header -->
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span
                    class="text-lg font-bold {didWin ? 'text-[var(--green)]' : 'text-[var(--red)]'}"
                >
                    {didWin ? "Victory" : "Defeat"}
                </span>
                {#if matchRecord.eloChange}
                    <span class="text-xs font-bold {matchRecord.eloChange > 0 ? 'text-green' : 'text-red'}">({matchRecord.eloChange > 0 ? '+' : ''}{matchRecord.eloChange})</span>
                {/if}
                <span class="text-xs text-muted-foreground">|</span>
                <span class="text-xs text-muted-foreground">{formatDate(matchRecord?.date)}</span>
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <span class="rounded border border-border px-1.5 py-0.5">{matchRecord?.betSize}{suitToSymbol.get(matchRecord?.trumpSuit)}</span>
                <span>{matchRecord?.botDifficulty}</span>
            </div>
        </div>

        <!-- Score display -->
        <div class="rounded-lg w-auto border border-border bg-card/60 p-3 text-sm">
            <ScoreDisplay matchRecord={matchRecord ?? {}} />
        </div>

        <!-- Played cards per player (in play order, with WonSet crowns) -->
        {#if matchRecord}
        <div class="flex flex-col gap-3 rounded-lg w-auto border border-border bg-card/40 p-3 text-sm">
            {#each playerPlayedCards as playedCards, playerID}
                {@const pid = playerID + 1}
                {@const colorVar = pid === 1 ? '--red' : pid === 2 ? '--blue' : pid === 3 ? '--yellow' : '--green'}
                {@const playerMeta = playersMeta[playerID]}
                {@const playerName = playerMeta?.username ?? `P${pid}`}
                {@const sets = matchRecord[`player${pid}Sets` as keyof typeof matchRecord] as number}
                <div class="p-3">
                    <div class="flex items-center gap-2 mb-4 text-xs">
                        <span class="font-medium" style="color: var({colorVar})">{playerName}</span>
                        <span class="text-muted-foreground">{sets} sets</span>
                        <span class="text-muted-foreground/50">| {playedCards.length} cards played</span>
                    </div>
                    <div class="flex flex-wrap gap-1.5">
                        {#each playedCards as card}
                            <PokerCard card={card} isIllegal={false} minify={true} />
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
        {/if}
    </div>
</div>
