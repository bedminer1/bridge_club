<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import Minus from "@lucide/svelte/icons/minus";
    import Plus from "@lucide/svelte/icons/plus";
    import CircleX from "@lucide/svelte/icons/circle-x";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { suitToSymbol } from "$lib/utils"
    import { findStrongestSuit } from "$lib/game/bot";
    import { wsClient } from "$lib/game/ws-client";
    import { playerIDToColor, playerName } from "$lib/game/player-utils";
    import type { Game } from "$lib/game/types"

    const FRONTEND_SUIT_TO_API: Record<string, string> = {
        Club: "Clubs", Diamond: "Diamonds", Heart: "Hearts", Spades: "Spades",
    }
    const SUIT_PRIORITY: Record<string, number> = {
        Club: 0, Diamond: 1, Heart: 2, Spades: 3,
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
    let strongestSuit: string | null = $state(null)
    let initializedBidTurn: string | null = $state(null)

    /** Returns the highest non-pass bid recorded in the current auction. */
    function highestBidSoFar(): { level: number, suit: string } {
        return game.Moves.reduce((highest, move) => {
            const bid = move.CardPlayed
            if (
                bid.Value > highest.level ||
                (bid.Value === highest.level && SUIT_PRIORITY[bid.Suit] > SUIT_PRIORITY[highest.suit])
            ) {
                return { level: bid.Value, suit: bid.Suit }
            }
            return highest
        }, { level: 0, suit: "Club" })
    }

    /** Returns whether this bid beats the highest bid recorded in the auction. */
    function isBidLegal(level: number, suit: string): boolean {
        const highestBid = highestBidSoFar()
        return level >= 1 && level <= 7 && (
            level > highestBid.level ||
            (level === highestBid.level && SUIT_PRIORITY[suit] > SUIT_PRIORITY[highestBid.suit])
        )
    }

    /** Returns the lowest bid level that beats the current bid in `suit`. */
    function minimumBidToBeatCurrent(suit: string): number {
        const highestBid = highestBidSoFar()
        if (highestBid.level === 0) return 1
        return isBidLegal(highestBid.level, suit)
            ? highestBid.level
            : highestBid.level + 1
    }

    /** Set the suggested suit and minimum legal level once for each human bid turn. */
    $effect(() => {
        const player = game.Players?.find(player => player.ID === humanPlayerId)
        const bidTurn = `${game.WhoseTurn}:${game.Moves?.length ?? 0}`
        if (player && !strongestSuit) strongestSuit = findStrongestSuit(player)
        if (!player || !strongestSuit || game.WhoseTurn !== humanPlayerId || initializedBidTurn === bidTurn) return

        bettedSuit = strongestSuit
        betSize = Math.min(minimumBidToBeatCurrent(strongestSuit), 7)
        initializedBidTurn = bidTurn
    })

    const isCurrentBidLegal = $derived(isBidLegal(betSize, bettedSuit))

    function raiseBet() {
        if (!roomId || disabled || !isCurrentBidLegal) return
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
            <div class="flex flex-1 items-center justify-center gap-2">
                <Button
                    variant="outline"
                    size="icon"
                    class="size-9 cursor-pointer disabled:blur-[1px]"
                    aria-label="Decrease bid"
                    onclick={() => betSize--}
                    disabled={disabled || betSize <= 1}
                >
                    <Minus class="size-5" />
                </Button>
                <Input
                    value={betSize}
                    readonly
                    aria-label="Bid level"
                    class="text-center numberInput w-12 px-1"
                />
                <Button
                    variant="outline"
                    size="icon"
                    class="size-9 cursor-pointer disabled:blur-[1px]"
                    aria-label="Increase bid"
                    onclick={() => betSize++}
                    disabled={disabled || betSize >= 7}
                >
                    <Plus class="size-5" />
                </Button>
            </div>
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
            <!-- {#if !isCurrentBidLegal}
                <CircleX class="size-9 shrink-0 self-center text-destructive" aria-label="This bid does not beat the current bid" />
            {/if} -->
            <Button 
            variant="destructive"
            onclick={raiseBet}
            disabled={disabled || !isCurrentBidLegal}
            class="bg-red-500 hover:!bg-red-600 cursor-pointer flex-1 disabled:cursor-not-allowed disabled:opacity-50"
            >Raise</Button>
        </div>
    </div>
</div>
