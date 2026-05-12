<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js";

    let { card, isIllegal }: { card: Card, isIllegal: boolean } = $props()

    const suitToSymbol = new Map<string, string>([
         ["Club", "♣"],
        ["Diamond", "♦"],
        ["Heart", "♥"],
        ["Spades", "♠"],
    ])

    const suitToColor = new Map<string, string>([
        ["Club", "black"],
        ["Diamond", "red-500"],
        ["Heart", "red-500"],
        ["Spades", "black"],
    ])

    const symbol = $derived(suitToSymbol.get(card.Suit))
    const color = $derived(suitToColor.get(card.Suit))
</script>

<Card.Root class="w-[50px] h-[60px] rounded-sm p-0 {isIllegal ? "bg-zinc-50 cursor-not-allowed brightness-70" : "bg-zinc-50 cursor-grab"}">
  <Card.Content class={`p-0 text-lg text-${color}`}>
    <p>{card.Rank} {symbol}</p>
  </Card.Content>
</Card.Root>

<!-- for safelist -->
<span class="text-red-500"></span>
<span class="text-black"></span>