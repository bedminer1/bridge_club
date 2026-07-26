<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { suitToSymbol } from "$lib/utils"
    import { isLegalRaise } from "$lib/game/betting";
    import { wsClient } from "$lib/game/ws-client";
    import { playerIDToColor, playerName } from "$lib/game/player-utils";
    import type { Game } from "$lib/game/types"

    const FRONTEND_SUIT_TO_API: Record<string, string> = {
        Club: "Clubs", Diamond: "Diamonds", Heart: "Hearts", Spades: "Spades",
    }

    let {
        game = {} as Game,
        humanSeat = 0,
        humanPlayerId = 0,
        hiddenMode = true,
        disabled = false,
        roomId = "",
    } = $props()

    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")

    // ── Default bet suggestions ──────────────────────────────────────

    const SUITS = ["Club", "Diamond", "Heart", "Spades"]

    /** Compute suit with highest total value (2=1pt, Ace=13pts) in player's hand. */
    let bestSuit = $derived.by(() => {
        const hand = game.Players?.[humanSeat]?.Cards ?? []
        const totals: Record<string, number> = { Club: 0, Diamond: 0, Heart: 0, Spades: 0 }
        for (const c of hand) { totals[c.Suit] = (totals[c.Suit] || 0) + (c.Value - 1) }
        return SUITS.reduce((best, s) => totals[s] > totals[best] ? s : best, "Club")
    })

    /** Minimum bid level to beat the current highest bid. 1 when no bid exists. */
    let minBet = $derived.by(() => {
        const lastBid = [...(game.Moves ?? [])].reverse().find((m: any) => m.CardPlayed.Value > 0)
        if (!lastBid) return 1
        const bid = lastBid.CardPlayed
        // Need higher level, or same level + higher suit
        const suitRank = SUITS.indexOf(bid.Suit ?? "Club")
        const ourRank = SUITS.indexOf(bettedSuit)
        if (ourRank > suitRank) return bid.Value  // same level, higher suit
        return Math.min(bid.Value + 1, 7)         // next level
    })

    // Auto-set defaults when it's our turn to bet
    $effect(() => {
        if (game.WhoseTurn === humanPlayerId && !disabled) {
            bettedSuit = bestSuit
            betSize = minBet
        }
    })

    function raiseBet() {
        if (!roomId || disabled) return
        const call = { Bid: { level: betSize, strain: FRONTEND_SUIT_TO_API[bettedSuit] ?? bettedSuit } }
        wsClient.gameAction("bid", call)
    }

    function passBet() {
        if (!roomId || disabled) return
        wsClient.gameAction("bid", "Pass")
    }
</script>

<!-- Moves display (last 3 betting actions) -->
<div class="flex flex-col items-center gap-1">
    {#each game.Moves.slice(-3) as move}
        <div class="flex items-center gap-2 {move === game.Moves[game.Moves.length - 1] ? 'text-base font-medium' : 'text-xs text-muted-foreground/60'}">
            <span class="text-{playerIDToColor.get(move.PlayerID)}">{playerName(game, move.PlayerID)}</span>
            {#if move.CardPlayed.Value === 0}
                <span>passed</span>
            {:else}
                <span>raised <strong>{move.CardPlayed.Value} {move.CardPlayed.Suit}</strong></span>
            {/if}
        </div>
    {/each}
    {#if game.Moves.length < 4}
        <div class="text-xs text-muted-foreground/40 mt-1">
            waiting for {playerName(game, game.WhoseTurn)}...
        </div>
    {/if}
    {#if game.Moves.length === 0}
        <div class="text-xs text-muted-foreground/40">
            {playerName(game, game.WhoseTurn)} to bet
        </div>
    {/if}
</div>

<!-- Player hands during betting -->
<div class="flex flex-col gap-10">
    {#each hiddenMode ? [game.Players[humanSeat]] : game.Players as player}
    <div class="flex flex-col h-[100px]">
        <p class="mb-2 text-{playerIDToColor.get(player.ID)}">{player.Username}</p>
        <div class="flex pl-4">
            {#each !hiddenMode || player.ID === humanPlayerId ? player.Cards : []  as card, index}
                <HandDisplay index={index}>
                    <PokerCard card={card} isIllegal={false} minify={false}/>
                </HandDisplay>
            {/each}
        </div>
    </div>
    {/each}
</div>

<!-- Raise / Pass controls -->
<div class="flex flex-col justify-center gap-2">
    <div class="flex flex-col gap-2 items-start w-[45%]">
        <div class="flex gap-2 w-full">
            <Input bind:value={betSize} class="text-center numberInput flex-1" type="number" min={1} max={7} placeholder="1-7"/>
            <Select.Root type="single" bind:value={bettedSuit}>
            <Select.Trigger class="flex-[3]">
                <p class="text-sm">{suitToSymbol.get(bettedSuit)} {bettedSuit}</p>
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="Club">&clubs; Club</Select.Item>
                <Select.Item value="Diamond">&diams; Diamond</Select.Item>
                <Select.Item value="Heart">&hearts; Heart</Select.Item>
                <Select.Item value="Spades">&spades; Spades</Select.Item>
            </Select.Content>
            </Select.Root>
        </div>
        <div class="flex gap-2 w-full">
            <Button class="cursor-pointer flex-1" onclick={passBet} disabled={disabled}>Pass</Button>
            <Button 
            variant="destructive"
            onclick={raiseBet}
            disabled={disabled || !isLegalRaise(game, betSize, bettedSuit)}
            class="bg-red-500 cursor-pointer flex-1 disabled:cursor-not-allowed disabled:opacity-50"
            >Raise</Button>
        </div>
    </div>
</div>
