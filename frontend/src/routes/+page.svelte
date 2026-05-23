<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";

    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    
    import { suitToSymbol } from "$lib/utils"
    import { isLegalRaise } from "$lib/game/betting";
    import { isCardIllegal } from "$lib/game/legality";
    import { headerState } from "$lib/game/header-state.svelte";

    import {
        createOnlineGame,
        getRoomState,
        doAdvance,
        doBid,
        doPlay,
        doSelectPartner,
        apiStateToGame,
    } from "$lib/game/api-game";
    import { page } from "$app/state";

    let { data } = $props()
    let { username, userID, token } = $state(data)

    // Online-only game state
    let isOnline = $state(false)
    let isOnlineLoading = $state(false)
    let game: any = $state({})
    let roomId = $state("")
    let onlineToken = $state(token ?? "")

    // user info
    let loggedIn: boolean = $derived(userID === 0 ? false : true)

    // Auto-start online game or join existing room when logged in
    $effect(() => {
        if (loggedIn && onlineToken && !isOnline && !isOnlineLoading) {
            // Check if we have a roomId from URL (lobby flow)
            const urlRoomId = page.url.searchParams.get("room")
            if (urlRoomId) {
                loadExistingRoom(urlRoomId)
            } else {
                startOnlineGame()
            }
        }
    })

    function onlogout() {
        loggedIn = false
    }

    // Determine which seat this user is in (for multiplayer)
    let humanSeat = $derived.by(() => {
        const s = page.url.searchParams.get("seat")
        return s !== null ? parseInt(s) : 0
    })
    let humanPlayerId = $derived(humanSeat + 1)

    // Sync reactive game state to shared header state
    $effect(() => { headerState.game = game })

    let isLightMode = $state(false)
    $effect(() => { headerState.isLightMode = isLightMode })

    // Sync user info to header
    $effect(() => { headerState.username = username })
    $effect(() => { headerState.loggedIn = loggedIn })

    // form inputs
    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")

    const suitOrder: Record<string, number> = { Spades: 0, Heart: 1, Club: 2, Diamond: 3 }
    let remainingDeck = $derived(
        game.FullDeck
            ?.filter((fc: any) => !game.Players?.[humanSeat]?.Cards?.some((pc: any) => pc.Suit === fc.Suit && pc.Value === fc.Value))
            .sort((a: any, b: any) => {
                const suitDiff = (suitOrder[a.Suit] ?? 0) - (suitOrder[b.Suit] ?? 0)
                if (suitDiff !== 0) return suitDiff
                return a.Value - b.Value
            }) ?? []
    )

    const playerIDToColor = new Map<number, string>([
        [1, "[var(--red)]"],
        [2, "[var(--blue)]"],
        [3, "[var(--yellow)]"],
        [4, "[var(--green)]"],
    ])

    // ── Online Game Actions ───────────────────────────────────────

    async function startOnlineGame() {
        if (!onlineToken) return
        isOnlineLoading = true
        try {
            const result = await createOnlineGame(username, headerState.difficulty, onlineToken)
            roomId = result.roomId
            game = result.game
            isOnline = true
        } catch (e) {
            console.error("Failed to start online game:", e)
            alert("Failed to start online game. Is the backend running at http://127.0.0.1:3000?")
        } finally {
            isOnlineLoading = false
        }
    }

    /** Load an existing room's state (from lobby flow). */
    async function loadExistingRoom(existingRoomId: string) {
        if (!onlineToken) return
        isOnlineLoading = true
        try {
            roomId = existingRoomId
            const gameState = await getRoomState(existingRoomId, onlineToken)
            fixupPlayerDisplay(gameState)
            game = gameState
            isOnline = true
            // Start polling if it's not the human's turn
            if (game.WhoseTurn !== humanPlayerId && game.Winner === "") {
                startPolling()
            }
        } catch (e) {
            console.error("Failed to load existing room:", e)
            alert("Failed to load game state for this room.")
        } finally {
            isOnlineLoading = false
        }
    }

    /** Poll timeout handle for online bot turn waiting. */
    let pollTimeout: ReturnType<typeof setTimeout> | null = null

    function startPolling() {
        stopPolling()
        _pollActive = true
        const delay = (headerState.botSpeed ?? 2) * 1000
        pollTimeout = setTimeout(() => {
            if (_pollActive) tick()
        }, delay)
    }

    let _pollActive = $state(false)

    async function tick() {
        if (!isOnline || !roomId || !onlineToken) {
            stopPolling()
            return
        }
        try {
            const updated = await doAdvance(roomId, onlineToken)
            fixupPlayerDisplay(updated)
            game = updated
            if (updated.WhoseTurn === humanPlayerId || updated.Winner !== "") {
                _pollActive = false
                return
            }
        } catch (e) {
            console.error("Poll error:", e)
            _pollActive = false
            return
        }
        // Schedule next tick with current bot speed (reads fresh each tick)
        const delay = (headerState.botSpeed ?? 2) * 1000
        pollTimeout = setTimeout(() => {
            if (_pollActive) tick()
        }, delay)
    }

    function stopPolling() {
        _pollActive = false
        if (pollTimeout !== null) {
            clearTimeout(pollTimeout)
            pollTimeout = null
        }
    }

    // React to bot speed changes during active polling
    let _lastBotSpeed = $state(headerState.botSpeed)
    $effect(() => {
        const current = headerState.botSpeed
        if (_pollActive && current !== _lastBotSpeed) {
            _lastBotSpeed = current
            stopPolling()
            _pollActive = true
            tick()
        }
        _lastBotSpeed = current
    })

    async function onlineRaiseBet(bs: number, suit: string) {
        if (!isOnline || !roomId || !onlineToken) return
        // Sync state first
        await syncState()
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            const call = { Bid: { level: bs, strain: FRONTEND_SUIT_TO_API[suit] ?? suit } }
            const updated = await doBid(roomId, onlineToken, call)
            fixupPlayerDisplay(updated)
            game = updated
            if (game.WhoseTurn !== humanPlayerId && game.Winner === "") {
                startPolling()
            }
        } catch (e) {
            console.error("Online raise failed:", e)
        }
    }

    async function onlinePassBet() {
        if (!isOnline || !roomId || !onlineToken) return
        await syncState()
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            const updated = await doBid(roomId, onlineToken, "Pass")
            fixupPlayerDisplay(updated)
            game = updated
            if (game.WhoseTurn !== humanPlayerId && game.Winner === "") {
                startPolling()
            }
        } catch (e) {
            console.error("Online pass failed:", e)
        }
    }

    async function onlineSelectPartner(card: any) {
        if (!isOnline || !roomId || !onlineToken) return
        await syncState()
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            const updated = await doSelectPartner(roomId, onlineToken, card)
            fixupPlayerDisplay(updated)
            game = updated
            if (game.WhoseTurn !== humanPlayerId && game.Winner === "") {
                startPolling()
            }
        } catch (e) {
            console.error("Online partner select failed:", e)
        }
    }

    async function onlinePlayCard(card: any, _player: any) {
        if (!isOnline || !roomId || !onlineToken) return
        await syncState()
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            const updated = await doPlay(roomId, onlineToken, card)
            fixupPlayerDisplay(updated)
            game = updated
            if (game.WhoseTurn !== humanPlayerId && game.Winner === "") {
                startPolling()
            }
        } catch (e) {
            console.error("Online play failed:", e)
        }
    }

    /** Re-sync game state from the backend to avoid acting on stale data. */
    async function syncState() {
        try {
            const res = await fetch(`${API_URL}/room/${roomId}/state`, {
                headers: { "X-Session-Token": onlineToken },
            })
            if (res.ok) {
                const data = await res.json()
                const betWinnerIdx = data.betWinner ?? undefined
                const updated = apiStateToGame(data, roomId, betWinnerIdx)
                fixupPlayerDisplay(updated)
                game = updated
            }
        } catch (e) {
            console.error("Sync error:", e)
        }
    }

    /** Fix up player display: mark the correct seat as human. */
    function fixupPlayerDisplay(g: any) {
        if (!g.Players) return
        const isMultiplayer = page.url.searchParams.has("seat")
        for (let i = 0; i < g.Players.length; i++) {
            g.Players[i].IsBot = i !== humanSeat
            g.Players[i].Username = i === humanSeat ? "You" : isMultiplayer ? `P${i + 1}` : `Bot ${i + 1}`
            g.Players[i].ShortUsername = i === humanSeat ? "Y" : `P${i + 1}`
        }
    }

    // Cleanup polling on component destroy
    $effect(() => {
        return () => stopPolling()
    })

    // Win detection — auto-save match immediately
    $effect(() => {
        if (isOnline && game.Winner !== "" && loggedIn && game.Players) {
            autoSaveMatch()
        }
    })

    const FRONTEND_SUIT_TO_API: Record<string, string> = {
        Club: "Clubs",
        Diamond: "Diamonds",
        Heart: "Hearts",
        Spades: "Spades",
    }

    const API_URL = "http://127.0.0.1:3000"

    /** Build each player's sequence of played cards from completed sets. */
    function playedCardsFromSets(sets: Array<{ Cards: Card[]; PlayerIDs: number[]; WinnerID: number }>): Card[][] {
        const played: Card[][] = [[], [], [], []]
        for (const set of sets) {
            for (let i = 0; i < set.Cards.length; i++) {
                const pid = (set.PlayerIDs[i] ?? 1) - 1
                played[pid].push(set.Cards[i])
            }
        }
        return played
    }

    // Save-time data: computed once when game ends
    let savePlayedCards: Card[][] = $derived(playedCardsFromSets(game.CompletedSets ?? []))
    let saveCompletedSetsJson: string = $derived(JSON.stringify(game.CompletedSets ?? []))

    /** Auto-save the match to the backend when the game ends. */
    async function autoSaveMatch() {
        const userTeam = game.Team1?.some((p: any) => p.ID === humanPlayerId) ? game.Team1 : game.Team2
        const wonMatch = game.Winner === "Team 1" && userTeam === game.Team1 ||
                         game.Winner === "Team 2" && userTeam === game.Team2 ? 1 : 0
        const partner = userTeam?.find((p: any) => p.ID !== humanPlayerId)?.ID ?? 0

        const body: Record<string, unknown> = {
            date: Date.now(),
            botDifficulty: headerState.difficulty,
            trumpSuit: game.Trump,
            betSize: game.BetSize,
            betWinner: game.BetWinner?.ID ?? 0,
            partner,
            wonMatch,
            setsData: saveCompletedSetsJson,
        }

        // Player sets counts
        for (let i = 0; i < 4; i++) {
            body[`player${i + 1}Sets`] = game.Players[i]?.Sets ?? 0
        }

        // Player played cards
        for (let i = 0; i < 4; i++) {
            body[`player${i + 1}Hand`] = JSON.stringify(savePlayedCards[i] ?? [])
        }

        try {
            const res = await fetch(`http://127.0.0.1:3000/api/matches`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "X-Session-Token": onlineToken,
                },
                body: JSON.stringify(body),
            })
            if (!res.ok) {
                console.error("Auto-save failed:", res.status, await res.text().catch(() => ""))
            }
        } catch (e) {
            console.error("Auto-save error:", e)
        }
    }
</script>

<div class="flex flex-col gap-6 w-full min-h-screen items-center px-4 pt-20 pb-8">

    {#if isOnlineLoading}
    <div class="text-lg text-muted-foreground animate-pulse">
        Starting game...
    </div>
    {:else if isOnline && game.Players}
    <div class="text-2xl text-muted-foreground">
        <p>Player {game.WhoseTurn}'s turn</p>
    </div>

    {#if game.IsPartnerSelectionPhase}
        {#if game.BetWinner.ID === humanPlayerId}
        <div class="flex flex-col gap-4 items-center">
            <p class="text-xl">Select a partner card</p>
            <p class="text-sm opacity-70">Choose any card you don't own — the player holding it becomes your partner</p>
            <div class="flex flex-wrap gap-1 justify-center max-w-3xl">
                {#each remainingDeck as card}
                    <button onclick={() => onlineSelectPartner(card)}
                        class="transition-transform brightness-105 dark:brightness-95 hover:brightness-130 dark:hover:brightness-120 hover:shadow-accent hover:shadow-xl/30 hover:-translate-y-1 active:brightness-125 active:shadow-accent rounded-sm">
                        <PokerCard card={card} isIllegal={false} minify={true} />
                    </button>
                {/each}
            </div>
        </div>
        {:else}
        <div class="flex flex-col gap-4 items-center">
            <p class="text-xl">Player {game.BetWinner.ID} is selecting a partner...</p>
        </div>
        {/if}
    {:else}
    <!-- Play area table -->
    <div class="flex flex-col gap-4 w-full max-w-3xl">
        <!-- Game info strip -->
        <div class="flex flex-nowrap gap-x-2 sm:gap-x-4 text-2xs sm:text-xs text-muted-foreground px-1 overflow-x-auto scrollbar-none">
            {#if game.BetSize > 0 || !game.IsBettingPhase}
            <span>Trump <strong class="text-accent font-medium">{suitToSymbol.get(game.Trump)} {game.Trump}</strong></span>
            <span>Bet <strong class="text-foreground font-medium">{game.BetSize}</strong></span>
            {/if}
            {#if game.IsBettingPhase && game.BetSize > 0}
            <span class="text-muted-foreground/40">|</span>
            <span class="whitespace-nowrap">Bet winner <strong class="text-foreground font-medium">P{game.BetWinner.ID}</strong> + partner need <strong class="text-accent font-medium">{6 + game.BetSize}</strong> sets</span>
            <span class="whitespace-nowrap">Opponents need <strong class="text-foreground font-medium">{8 - game.BetSize}</strong> sets</span>
            {/if}
            {#if !game.IsBettingPhase}
            <span>Winner <strong class="text-foreground font-medium">P{game.BetWinner.ID}</strong></span>
            <span>Partner <strong class="text-accent font-medium">{game.PartnerCard.Rank}{suitToSymbol.get(game.PartnerCard.Suit)}</strong></span>
            <span class="text-muted-foreground">|</span>
            <span>Set <strong class="text-foreground font-medium">{game.Players.reduce((s: number, p: any) => s + p.Sets, 0)}/13</strong></span>
            {/if}
        </div>

    <div class="rounded-xl border border-border bg-card/50 p-4 sm:p-6">
    {#if game.IsBettingPhase}
    <div class="flex flex-col items-center gap-1">
        {#each game.Moves.slice(-3) as move}
            <div class="flex items-center gap-2 {move === game.Moves[game.Moves.length - 1] ? 'text-base font-medium' : 'text-xs text-muted-foreground/60'}">
                <span class="text-{playerIDToColor.get(move.PlayerID)}">P{move.PlayerID}</span>
                {#if move.CardPlayed.Value === 0}
                    <span>passed</span>
                {:else}
                    <span>raised <strong>{move.CardPlayed.Value} {move.CardPlayed.Suit}</strong></span>
                {/if}
            </div>
        {/each}
        {#if game.Moves.length < 4}
            <div class="text-xs text-muted-foreground/40 mt-1">
                waiting for P{game.WhoseTurn}...
            </div>
        {/if}
        {#if game.Moves.length === 0}
            <div class="text-xs text-muted-foreground/40">
                P{game.WhoseTurn} to bet
            </div>
        {/if}
    </div>
    {:else}
    <div class="flex justify-center relative h-21 w-full">
        <div class="flex gap-2 mx-auto">
            {#each game.Moves as move}
            <div class="flex flex-col items-center">
                <PokerCard card={move.CardPlayed} isIllegal={false} minify={false} />
                <p class="text-{playerIDToColor.get(move.PlayerID)}">P{move.PlayerID}</p>
            </div>
            {/each}
        </div>
    
        {#if game.PreviousMoves.length !== 0} 
        <div class="absolute right-2 bottom-2 sm:right-1/6 sm:bottom-1/4 flex pl-4">
            {#each game.PreviousMoves as move, index}
                <HandDisplay index={index}>
                    <PokerCard card={move.CardPlayed} isIllegal={false} minify={true} />
                    <p class="text-xs text-{playerIDToColor.get(move.PlayerID)}">P{move.PlayerID}</p>
                </HandDisplay>
            {/each}
        </div>
        {/if}
    </div>
    {/if}


    {#if !game.IsBettingPhase}
    <!-- MAIN PHASE -->
    <div class="flex flex-col gap-10">
        {#each headerState.hiddenMode ? [game.Players[humanSeat]] : game.Players as player}
        <div>
            <div class="flex gap-2">
                <p class="text-{playerIDToColor.get(player.ID)}">{player.Username} ({player.Sets} sets) </p>
                {#if !headerState.hiddenMode && player.Partner !== null}
                <p>| Partner is Player {player.Partner?.ID}</p>
                {/if}
            </div>
            
            <div class="flex h-[100px] pl-4">
                {#each player.Cards  as card, index}
                <button
                    disabled={isCardIllegal(game, player, card)}
                    onclick={() => onlinePlayCard(card, player)}
                    class="text-left">
                    <HandDisplay index={index}>
                        <PokerCard card={card} isIllegal={isCardIllegal(game, player, card)} minify={false}/>
                    </HandDisplay>
                </button>
                {/each}
                {#if !headerState.hiddenMode}
                <Separator orientation="vertical" class="mx-10 h-full"/>
                    {#each player.PlayedCards as card, index}
                     <button 
                        disabled={true}>
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={true} minify={false}/>
                        </HandDisplay>
                    </button>
                    {/each}
                {/if}
            </div>
        </div>
        {/each}

        {#if headerState.hiddenMode}
        <div class="flex gap-4">
            {#each game.Players.slice(1, 4) as player, index}
                <div class="flex gap-2">
                <p class="text-{playerIDToColor.get(player.ID)}">Player {player.ID} ({player.Sets} sets) </p>
                </div>

                {#if index < 2}
                <Separator orientation="vertical" />
                {/if}
            {/each}
        </div>
        {/if}
    </div>


    {:else} 
    <!-- BETTING PHASE -->
        <div class="flex flex-col gap-10">
            {#each headerState.hiddenMode ? [game.Players[humanSeat]] : game.Players as player}
            <div class="flex flex-col h-[100px]">
                <p class="mb-2 text-{playerIDToColor.get(player.ID)}">{player.Username}</p>
                <div class="flex pl-4">
                    {#each !headerState.hiddenMode || player.ID === humanPlayerId ? player.Cards : []  as card, index}
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={false} minify={false}/>
                        </HandDisplay>
                    {/each}
                </div>
            </div>
            {/each}
        </div>

        <div class="flex flex-col justify-center gap-2">
            <div class="flex flex-col gap-2 items-start w-[45%]">
                <div class="flex gap-2 w-full">
                    <Input bind:value={betSize} class="text-center numberInput flex-1" type="number" min={1} max={7} placeholder="1-7"/>
                    <Select.Root type="single" bind:value={bettedSuit}>
                    <Select.Trigger class="flex-[3]">
                        <p class="text-sm">{suitToSymbol.get(bettedSuit)} {bettedSuit}</p>
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="Club">♣ Club</Select.Item>
                        <Select.Item value="Diamond">♦ Diamond</Select.Item>
                        <Select.Item value="Heart">♥ Heart</Select.Item>
                        <Select.Item value="Spades">♠ Spades</Select.Item>
                    </Select.Content>
                    </Select.Root>
                </div>
                <div class="flex gap-2 w-full">
                    <Button class="flex-1" onclick={onlinePassBet}>Pass</Button>
                    <Button 
                    variant="destructive"
                    onclick={() => onlineRaiseBet(betSize, bettedSuit)}
                    disabled={!isLegalRaise(game, betSize, bettedSuit)}
                    class="flex-1"
                    >Raise</Button>
                </div>
            </div>
        </div>
    {/if}
        </div>
    </div>
    {/if}

{/if}
</div>
