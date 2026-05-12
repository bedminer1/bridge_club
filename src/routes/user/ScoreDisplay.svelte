<script lang="ts">
    let { matchRecord }: { matchRecord: MatchRecord} = $props()

    const partner = matchRecord.partner
    const opponents = [2, 3, 4].filter(n => n !== matchRecord.partner)

    function getSets(player: number): number {
        return matchRecord[`player${player}Sets` as keyof MatchRecord] as number
    }
</script>

<p>
    <span class="text-amber-200">P1[{getSets(1)}]</span> |
    <span class="text-blue-300">P{partner}[{getSets(partner!)}]</span> :
    <span class="text-red-300">P{opponents[0]}[{getSets(opponents[0])}]</span> |
    <span class="text-red-300">P{opponents[1]}[{getSets(opponents[1])}]</span>
</p>