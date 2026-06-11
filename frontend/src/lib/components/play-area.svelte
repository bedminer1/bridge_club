<script lang="ts">
    import { Separator } from "$lib/components/ui/separator/index.js";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { isCardIllegal } from "$lib/game/legality";
    import { wsClient } from "$lib/game/ws-client";
    import { playerIDToColor, playerName } from "$lib/game/player-utils";
    import { frontendCardToApiCard } from "$lib/game/api-game";
    import type { Game } from "$lib/game/types"

    let {
        game = {} as Game,
        humanSeat = 0,
        humanPlayerId = 0,
        hiddenMode = true,
        disabled = false,
        roomId = "",
    } = $props()

    function playCard(card: any) {
        if (!roomId || disabled) return
        wsClient.gameAction("play", undefined, frontendCardToApiCard(card))
    }
</script>

<!-- Current trick cards -->
<div class="flex justify-between w-full min-h-28 relative">
    <div class="flex gap-1.5 sm:gap-2 ml-2 sm:ml-4 items-start flex-shrink-0">
        {#each game.Moves as move, i}
        <div class="flex flex-col {i % 2 === 0 ? 'items-center mb-8' : 'items-center mt-8'}">
            <PokerCard card={move.CardPlayed} isIllegal={false} minify={false} />
            <p class="text-{playerIDToColor.get(move.PlayerID)} text-2xs sm:text-xs whitespace-nowrap">{playerName(game, move.PlayerID)}</p>
        </div>
        {/each}
    </div>

    {#if game.PreviousMoves.length !== 0}
    <Separator orientation="vertical" class="mx-2" />
    <div class="flex gap-1.5 sm:gap-2 mr-2 sm:mr-4 items-start flex-shrink-0">
        {#each game.PreviousMoves as move, i}
        <div class="flex flex-col {i % 2 === 0 ? 'items-center mb-8' : 'items-center mt-8'}">
            <PokerCard card={move.CardPlayed} isIllegal={false} minify={true} />
            <p class="text-2xs text-right text-{playerIDToColor.get(move.PlayerID)} whitespace-nowrap">P{move.PlayerID}</p>
        </div>
        {/each}
    </div>
    {/if}
</div>

<div class="flex flex-wrap justify-center gap-6 text-xs">
    {#each game.Players as player}
        <p class="text-{playerIDToColor.get(player.ID)}">{player.Username} ({player.Sets} sets) </p>
    {/each}
</div>
<div class="flex flex-col gap-6 mt-6">
    {#each game.Players as player, playerI}
    <div class="flex flex-col mx-4">
        <p class="text-{playerIDToColor.get(player.ID)}">{player.Username}</p>
        <div class="flex gap-1 justify-center">
            {#if !hiddenMode || player.ID === humanPlayerId}
                {#each player.Cards as card, index}
                    <button onclick={() => playCard(card)}
                        disabled={disabled || player.ID !== humanPlayerId || isCardIllegal(game, player, card)}
                        class="transition-transform brightness-105 dark:brightness-95 hover:brightness-130 dark:hover:brightness-120 hover:shadow-accent hover:shadow-xl/30 hover:-translate-y-1 active:brightness-125 active:shadow-accent rounded-sm disabled:grayscale disabled:hover:translate-y-0 disabled:cursor-not-allowed">
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={isCardIllegal(game, player, card)} minify={false}/>
                        </HandDisplay>
                    </button>
                {/each}
            {:else}
                <div class="flex gap-1">
                    {#each Array(player.Cards.length) as _, _i}
                    <HandDisplay index={0}>
                        <div class="w-[43px] h-[52px] sm:w-[50px] sm:h-[60px] rounded-sm border border-indigo-800/40 bg-gradient-to-br from-slate-800 via-indigo-950 to-slate-900 flex items-center justify-center overflow-hidden shadow-inner">
                            <div class="w-4/5 h-[85%] rounded-[2px] border border-indigo-500/20 flex items-center justify-center bg-slate-950/40">
                                <span class="text-indigo-400/30 text-[11px] sm:text-sm font-bold">♠</span>
                            </div>
                        </div>
                    </HandDisplay>
                    {/each}
                </div>
            {/if}
        </div>
    </div>
    {/each}
</div>
