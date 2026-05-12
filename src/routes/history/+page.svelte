<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js"
    import ScoreDisplay from "./ScoreDisplay.svelte";
    import HandDisplay from "$lib/components/HandDisplay.svelte";
    import PokerCard from "$lib/components/PokerCard.svelte";

    let { data } = $props()
    let { matchRecords } = $state(data)

    const winRate = $state(0) // percentage

    function parseHand(handJson: string): Card[] {
        const hand = JSON.parse(handJson)
        return hand
    }

    function formatDate(timestamp: number): string {
        const date = new Date(timestamp)

        const day = String(date.getDate()).padStart(2, '0')
        const month = String(date.getMonth() + 1).padStart(2, '0') // months are 0-indexed
        const year = String(date.getFullYear()).slice(-2)

        const hours = String(date.getHours()).padStart(2, '0')
        const minutes = String(date.getMinutes()).padStart(2, '0')

        return `${day}/${month}/${year} ${hours}:${minutes}`
    }

</script>

<div class="flex flex-col w-full h-screen items-center pt-7">
    <div class="w-full p-4 flex flex-col gap-4 justify-center items-center">
        {#each matchRecords as matchRecord}
        <Card.Root class="w-full rounded-none flex-row gap-1 h-auto pb-3 pt-2">
            <Card.Header class="w-1/5 flex flex-col items-start pl-2 sm:pl-4 sm:w-1/4">
                <Card.Title class="text-2xl {matchRecord.wonMatch ? "text-blue-300" : "text-red-400"}">{matchRecord.wonMatch ? "Victory" : "Defeat"}</Card.Title>
                <Card.Description class="md:text-nowrap"> 
                    <span class="text-nowrap text-xs sm:text-base">{matchRecord.betSize} {matchRecord.trumpSuit.toUpperCase()} </span>
                    <span class="text-nowrap text-xs sm:text-base">| {matchRecord.botDifficulty.toUpperCase()}</span></Card.Description>
            </Card.Header>
            
            <Card.Content class="flex flex-col w-full sm:w-3/4 h-full sm:text-base text-xs pr-0">
                <ScoreDisplay matchRecord={matchRecord} />
                <div class="flex pl-4 mt-6">
                    {#each parseHand(matchRecord.player1Hand)  as card, index}
                       <HandDisplay index={index}>
                           <PokerCard card={card} isIllegal={false} minify={true}/>
                       </HandDisplay>
                   {/each}
                </div>
            </Card.Content>

             <Card.Footer class="hidden sm:flex w-1/5">
                <p>{formatDate(matchRecord.date)}</p>
            </Card.Footer>
        </Card.Root>
        {/each}
    </div>
</div>