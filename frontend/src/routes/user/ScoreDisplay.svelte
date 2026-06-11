<script lang="ts">
    import type { MatchRecord } from "$lib/game/types"

    let { matchRecord }: { matchRecord: MatchRecord } = $props()

    // Display Team 1 vs Team 2 scores
    let team1Sets = $derived(matchRecord?.team1Sets ?? 0)
    let team2Sets = $derived(matchRecord?.team2Sets ?? 0)
    let team1Won = $derived(matchRecord?.winningTeam === 1)
</script>

<p>
    <span class="text-[var(--blue)]">Team 1 [{team1Sets}]</span>
    :
    <span class="text-[var(--red)]">Team 2 [{team2Sets}]</span>
    {#if team1Won}
        <span class="text-blue text-xs ml-2">W</span>
    {:else}
        <span class="text-red text-xs ml-2">L</span>
    {/if}
</p>
