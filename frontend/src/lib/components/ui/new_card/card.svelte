<script lang="ts">
    import { onMount } from "svelte"
    import { StandardDeckOfCards } from "card-factory"

    let host: HTMLDivElement | null = null

    onMount(() => {
        if (!host) return

        const deck = StandardDeckOfCards()
        const pile = deck.createPileElement("drawPile", deck.cards)
        const discardPile = deck.createPileElement("discardPile");  
        const playerHand = deck.createPileElement("Hand"); // will begin with no cards
        host.replaceChildren()
        host.appendChild(pile.container)
        host.appendChild(discardPile.container)
        host.appendChild(playerHand.container)

        pile.container.addEventListener("click", () => {
            pile.topCardElement.flip();
            playerHand.applyCascadeLayout("cascade");
        });

        pile.cascade()
    })
</script>

<div bind:this={host} class="min-h-[120px] w-full overflow-x-auto rounded-md border border-border bg-muted/20 p-4"></div>