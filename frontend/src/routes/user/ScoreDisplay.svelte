<script lang="ts">
    let { matchRecord }: { matchRecord: MatchRecord} = $props()

    const partner = matchRecord.partner
    const opponents = [2, 3, 4].filter(n => n !== matchRecord.partner)

    function getSets(player: number): number {
        return matchRecord[`player${player}Sets` as keyof MatchRecord] as number
    }
</script>

<p>
    <span class="text-[var(--red)]">P1[{getSets(1)}]</span> |
    <span class="text-[var(--blue)]">P{partner}[{getSets(partner!)}]</span> :
    <span class="text-[var(--yellow)]">P{opponents[0]}[{getSets(opponents[0])}]</span> |
    <span class="text-[var(--green)]">P{opponents[1]}[{getSets(opponents[1])}]</span>
</p>