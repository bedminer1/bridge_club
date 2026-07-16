<script lang="ts">
    import { page } from "$app/state";
    import ScoreDisplay from "../ScoreDisplay.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import { formatDate, suitToSymbol } from "$lib/utils";

    let { data } = $props()
    let { matchRecord, userID } = $state(data)

    // Find the current user's participant entry
    let myParticipant = $derived(
        matchRecord?.participants?.find((p: any) => p.userId === userID)
    )

    // Determine win/loss for the viewer
    let didWin = $derived(
        myParticipant ? myParticipant.team === matchRecord.winningTeam : false
    )

    // My elo change
    let myElo = $derived(myParticipant?.eloChange ?? 0)

    // Parse each participant's played cards
    const participantsWithCards = $derived(
        (matchRecord?.participants ?? [])
            .slice()
            .sort((a: any, b: any) => a.seatIndex - b.seatIndex)
            .map((p: any) => ({
                ...p,
                cards: (() => {
                    return parsePreview(p.handPreview || p.cardsPlayed || "[]")
                })(),
            }))
    )

    function parsePreview(preview: string): Card[] {
        const suitMap: Record<string, string> = { c: "Club", d: "Diamond", h: "Heart", s: "Spades" }
        const rankToValue: Record<string, number> = {
            "2": 2, "3": 3, "4": 4, "5": 5, "6": 6, "7": 7, "8": 8, "9": 9,
            "10": 10, "J": 11, "Q": 12, "K": 13, "A": 14,
        }
        const valueToRank: Record<number, string> = {
            2: "2", 3: "3", 4: "4", 5: "5", 6: "6", 7: "7", 8: "8", 9: "9",
            10: "10", 11: "J", 12: "Q", 13: "K", 14: "A",
        }
        if (!preview || preview === "[]") return []
        // If it's JSON (cardsPlayed format), parse as JSON
        if (preview.startsWith("[")) {
            try { return JSON.parse(preview) } catch { return [] }
        }
        // Compact hand_preview format: "2cw3hlAdw..."
        const cards: Card[] = []
        let i = 0
        while (i < preview.length) {
            const isTen = preview[i] === "1" && preview[i + 1] === "0"
            const rankStr = isTen ? "10" : preview[i]
            const suitLetter = preview[isTen ? i + 2 : i + 1]
            const won = preview[isTen ? i + 3 : i + 2] === "w"
            const value = rankToValue[rankStr]
            if (value && suitLetter) {
                cards.push({
                    Rank: valueToRank[value] ?? rankStr,
                    Value: value,
                    Suit: suitMap[suitLetter] ?? "Club",
                    WonSet: won,
                })
            }
            i += isTen ? 4 : 3
        }
        return cards
    }

    const playerColor: Record<number, string> = {
        0: 'var(--red)',
        1: 'var(--blue)',
        2: 'var(--yellow)',
        3: 'var(--green)',
    }

    // Determine partner seat from match data
    let partnerSeat = $derived(matchRecord?.partnerIdx ?? null)

    function getPlayerName(participant: any): string {
        // We don't have usernames in the participant data directly from API
        // The frontend gets usernames from the /api/matches response
        // For now, use seat-based names
        const seat = participant.seatIndex
        if (participant.userId <= 3) {
            return ["Bot-Alpha", "Bot-Beta", "Bot-Gamma"][participant.userId - 1] || `Bot`
        }
        if (participant.userId === userID) {
            return "You"
        }
        return `P${seat + 1}`
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
                    class="text-lg font-bold {didWin ? 'text-[var(--blue)]' : 'text-[var(--red)]'}"
                >
                    {didWin ? "Win" : "Loss"}
                </span>
                {#if myElo !== 0}
                    <span class="text-xs font-bold {myElo > 0 ? 'text-green' : 'text-red'}">({myElo > 0 ? '+' : ''}{myElo})</span>
                {/if}
                <span class="text-xs text-muted-foreground">|</span>
                <span class="text-xs text-muted-foreground">{formatDate(matchRecord?.createdAt)}</span>
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <span class="rounded border border-border px-1.5 py-0.5">{matchRecord?.betSize}{suitToSymbol.get(matchRecord?.trumpSuit)}</span>
                <span>{matchRecord?.matchType === "single" ? "vs Bots" : "Multiplayer"}</span>
            </div>
        </div>

        <!-- Score display -->
        <div class="rounded-lg w-auto border border-border bg-card/60 p-3 text-sm">
            <ScoreDisplay matchRecord={matchRecord} />
        </div>

        <!-- Played cards per participant -->
        {#if matchRecord}
        <div class="flex flex-col gap-3 rounded-lg w-auto border border-border bg-card/40 p-3 text-sm">
            {#each participantsWithCards as participant}
                {@const seat = participant.seatIndex}
                <div class="p-3">
                    <div class="flex items-center gap-2 mb-4 text-xs">
                        <span class="font-medium" style="color: var(--red)">{getPlayerName(participant)}</span>
                        <span class="text-muted-foreground">{participant.setsWon} sets</span>
                        <span class="text-muted-foreground/50">| {participant.cards.length} cards played</span>
                        {#if participant.team === 1}
                            <span class="text-blue text-[10px]">Team 1</span>
                        {:else}
                            <span class="text-yellow text-[10px]">Team 2</span>
                        {/if}
                        {#if seat === matchRecord.betWinnerIdx}
                            <span class="text-accent text-[10px]">(bet winner)</span>
                        {/if}
                        {#if matchRecord.partnerIdx !== null && seat === matchRecord.partnerIdx}
                            <span class="text-accent text-[10px]">(partner)</span>
                        {/if}
                    </div>
                    <div class="flex flex-wrap gap-1.5">
                        {#each participant.cards as card}
                            <PokerCard card={card} isIllegal={false} minify={true} isHistory={true} />
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
        {/if}
    </div>
</div>
