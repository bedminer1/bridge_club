<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js"
    import ScoreDisplay from "../ScoreDisplay.svelte";
    import PokerCard from "$lib/components/PokerCard.svelte";
    import HandDisplay from "$lib/components/HandDisplay.svelte";
    import { formatDate } from "$lib/utils";

    let { data } = $props()
    let { matchRecord } = $state(data)

    const playerHands: Card[][] = []
    for (let i = 1; i <= 4; i++) {
        const cardsStr = matchRecord[`player${i}Hand` as keyof MatchRecord] as string
        const cards = JSON.parse(cardsStr)
        console.log(cards)
        playerHands.push(cards)
    }
</script>

<div class="pt-14 px-4">
    <Card.Root class="w-full rounded-none flex-row gap-1 h-auto pb-3 pt-2">
        <Card.Header class="w-1/5 flex flex-col items-start pl-2 sm:pl-4 sm:w-1/3">
            <Card.Title class="text-2xl {matchRecord.wonMatch ? "text-blue-300" : "text-red-400"}">{matchRecord.wonMatch ? "Victory" : "Defeat"}</Card.Title>
            <Card.Description class="md:text-nowrap"> 
                <span class="text-nowrap text-xs sm:text-base">{matchRecord.betSize} {matchRecord.trumpSuit.toUpperCase()} </span>
                <span class="text-nowrap text-xs sm:text-base">| {matchRecord.botDifficulty.toUpperCase()}</span></Card.Description>
        </Card.Header>
        
        <Card.Content class="flex flex-col w-full sm:w-3/4 h-full sm:text-base text-xs pr-0">
            <ScoreDisplay matchRecord={matchRecord} />
            <div>
                {#each playerHands as playerHand, playerID}
                <div class="flex pl-4 mt-6">
                        {#each playerHand as card, index}
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={false} minify={true}/>
                        </HandDisplay>
                        {/each}
                    </div>
                {/each}
            </div>
        </Card.Content>
    
        <Card.Footer class="hidden sm:flex w-1/5">
            <p>{formatDate(matchRecord.date)}</p>
        </Card.Footer>
    </Card.Root>
</div>