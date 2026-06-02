<script lang="ts">
    import { suitToSymbol } from "$lib/utils"
    import type { Game } from "$lib/game/types"

    let {
        game = {} as Game,
        humanSeat = 0,
        humanPlayerId = 0,
    } = $props()

    function playerName(playerId: number): string {
        return game.Players?.find((p: any) => p.ID === playerId)?.Username ?? `P${playerId}`
    }
</script>

<div class="flex flex-nowrap gap-x-2 sm:gap-x-4 text-2xs sm:text-xs text-muted-foreground px-1 overflow-x-auto scrollbar-none">
    {#if game.BetSize > 0 || !game.IsBettingPhase}
    <span>Trump <strong class="text-accent font-medium">{suitToSymbol.get(game.Trump)} {game.Trump}</strong></span>
    <span>Bet <strong class="text-foreground font-medium">{game.BetSize}</strong></span>
    {/if}
    {#if game.IsBettingPhase && game.BetSize > 0}
    <span class="text-muted-foreground/40">|</span>
    <span class="whitespace-nowrap"><strong class="text-foreground font-medium">{playerName(game.BetWinner.ID)}</strong> + partner need <strong class="text-accent font-medium">{6 + game.BetSize}</strong> sets</span>
    <span class="whitespace-nowrap">Opponents need <strong class="text-foreground font-medium">{8 - game.BetSize}</strong> sets</span>
    {/if}
    {#if !game.IsBettingPhase}
    <span>Winner <strong class="text-foreground font-medium">{playerName(game.BetWinner.ID)}</strong></span>
    <span>Partner <strong class="text-accent font-medium">{game.PartnerCard.Rank}{suitToSymbol.get(game.PartnerCard.Suit)}</strong></span>
    <span class="text-muted-foreground">|</span>
    <span>Set <strong class="text-foreground font-medium">{game.Players.reduce((s: number, p: any) => s + p.Sets, 0)}/13</strong></span>
    {/if}
</div>
