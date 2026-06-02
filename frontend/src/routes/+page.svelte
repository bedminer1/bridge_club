<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Settings } from "@lucide/svelte";

    import PokerCard from "$lib/components/poker-card.svelte";
    
    import { suitToSymbol } from "$lib/utils"
    import { headerState } from "$lib/game/header-state.svelte";

    import {
        createOnlineGame,
        getRoomState,
        apiStateToGame,
        frontendCardToApiCard,
    } from "$lib/game/api-game";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";

    import { wsClient } from "$lib/game/ws-client";

    // ── Import extracted components ─────────────────────────────────
    import Lobby from "$lib/components/lobby.svelte";
    import GameInfo from "$lib/components/game-info.svelte";
    import BidArea from "$lib/components/bid-area.svelte";
    import PlayArea from "$lib/components/play-area.svelte";
    import Chat from "$lib/components/chat.svelte";

    let { data } = $props()
    let { username, userID, token } = $state(data)

    // Online-only game state
    let isOnline = $state(false)
    let isOnlineLoading = $state(false)
    let game: any = $state({})
    let roomId = $state("")
    let onlineToken = $state(token ?? "")

    // Lock hidden mode toggle if it was on at game start
    let hiddenModeLocked = $state(false)

    // Match result data for the game-over dialog
    let lastMatchId = $state<number | null>(null)
    let lastEloChange = $state<number | null>(null)
    let loadingMatchResult = $state(false)

    // user info
    let loggedIn: boolean = $derived(userID === 0 ? false : true)

    // Load a room if the URL has a ?room= param; otherwise show the lobby
    $effect(() => {
        const urlRoomId = page.url.searchParams.get("room")
        if (urlRoomId) {
            // Save active room to localStorage so we can resume after navigation
            try { localStorage.setItem("bridgeActiveRoom", JSON.stringify({ roomId: urlRoomId, seat: page.url.searchParams.get("seat") })) } catch {}
            if (loggedIn && onlineToken && !isOnline && !isOnlineLoading) {
                loadExistingRoom(urlRoomId)
            }
        }
    })

    // Resume active room from localStorage when returning to page without ?room=
    $effect(() => {
        if (!page.url.searchParams.get("room") && loggedIn && onlineToken && !isOnline && !isOnlineLoading) {
            try {
                const saved = localStorage.getItem("bridgeActiveRoom")
                if (saved) {
                    const { roomId, seat } = JSON.parse(saved)
                    if (roomId) {
                        const params = new URLSearchParams()
                        params.set("room", roomId)
                        if (seat) params.set("seat", seat)
                        goto(`/?${params.toString()}`, { replaceState: true })
                    }
                }
            } catch {}
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

    // Game-over dialog
    let showGameOverDialog = $state(false)
    $effect(() => {
        if (game.Winner) {
            showGameOverDialog = true
        }
    })
    const team1Sets = $derived(game.Players?.reduce?.((s: number, p: any, i: number) => {
        const team1Ids = new Set(game.Team1?.map((t: any) => t.ID) ?? [])
        return s + (team1Ids.has(p.ID) ? (p.Sets ?? 0) : 0)
    }, 0) ?? 0)
    const team2Sets = $derived(game.Players?.reduce?.((s: number, p: any, i: number) => {
        const team2Ids = new Set(game.Team2?.map((t: any) => t.ID) ?? [])
        return s + (team2Ids.has(p.ID) ? (p.Sets ?? 0) : 0)
    }, 0) ?? 0)

    // Sync user info to header
    $effect(() => { headerState.username = username })
    $effect(() => { headerState.loggedIn = loggedIn })

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
            // Fetch room info for hidden mode setting
            try {
                const infoRes = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(existingRoomId)}/info`, {
                    headers: { "X-Session-Token": onlineToken },
                })
                if (infoRes.ok) {
                    const info = await infoRes.json()
                    headerState.hiddenMode = info.hiddenMode ?? true
                }
            } catch {}
        } catch (e) {
            console.error("Failed to load existing room:", e)
            try { localStorage.removeItem("bridgeActiveRoom") } catch {}
            // Redirect to lobby silently (room went away, e.g. server restart)
            if (page.url.searchParams.get("room")) {
                goto("/", { replaceState: true })
            }
        } finally {
            isOnlineLoading = false
        }
    }

    async function onlineRaiseBet(bs: number, suit: string) {
        if (!isOnline || !roomId || !onlineToken) return
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            const call = { Bid: { level: bs, strain: FRONTEND_SUIT_TO_API[suit] ?? suit } }
            wsClient.gameAction("bid", call)
        } catch (e) {
            console.error("Online raise failed:", e)
        }
    }

    async function onlinePassBet() {
        if (!isOnline || !roomId || !onlineToken) return
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            wsClient.gameAction("bid", "Pass")
        } catch (e) {
            console.error("Online pass failed:", e)
        }
    }

    async function onlineSelectPartner(card: any) {
        if (!isOnline || !roomId || !onlineToken) return
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            wsClient.gameAction("selectPartner", undefined, frontendCardToApiCard(card))
        } catch (e) {
            console.error("Online partner select failed:", e)
        }
    }

    async function onlinePlayCard(card: any, _player: any) {
        if (!isOnline || !roomId || !onlineToken) return
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            wsClient.gameAction("play", undefined, frontendCardToApiCard(card))
        } catch (e) {
            console.error("Online play failed:", e)
        }
    }

    /** Fix up player display: ensure IsBot is correct without overwriting real names. */
    function fixupPlayerDisplay(g: any) {
        if (!g.Players) return
        for (let i = 0; i < g.Players.length; i++) {
            g.Players[i].IsBot = i !== humanSeat
        }
    }

    // Utility: get a player's display name from their ID
    function playerName(playerId: number): string {
        return game.Players?.find((p: any) => p.ID === playerId)?.Username ?? `P${playerId}`
    }

    // Win detection — auto-save match immediately, clear saved room
    $effect(() => {
        if (isOnline && game.Winner !== "" && loggedIn && game.Players) {
            autoSaveMatch()
            try { localStorage.removeItem("bridgeActiveRoom") } catch {}
        }
    })

    const FRONTEND_SUIT_TO_API: Record<string, string> = {
        Club: "Clubs",
        Diamond: "Diamonds",
        Heart: "Hearts",
        Spades: "Spades",
    }

    let API_URL: string
    if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
        API_URL = "http://127.0.0.1:3000"
    } else {
        API_URL = "https://bridge-club.duckdns.org"
    }

    // ── Lobby State ─────────────────────────────────────────────
    let lobbyRoomId = $state("")
    let lobbyPlayerId = $state("")
    let lobbyMySeatIndex = $state(0)
    let lobbyIsHost = $state(false)
    let lobbyPlayers = $state<Array<{ name: string; seatIndex: number; isBot: boolean }>>([])
    let lobbyHiddenMode = $state(true)

    function lobbyCreateRoom() {
        try {
            wsClient.createLobby()
        } catch (e) { console.error("Create room error:", e); alert("Failed. Is the backend running?") }
    }

    function lobbyJoinRoom(joinRoomId: string) {
        if (!joinRoomId.trim()) return
        try {
            wsClient.joinLobby(joinRoomId.trim())
        } catch (e) { console.error("Join error:", e); alert("Failed. Is the backend running?") }
    }

    function lobbyLeaveRoom() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        try {
            wsClient.leaveLobby()
        } catch (e) { console.error("Leave error:", e) }
        roomId = ""; lobbyRoomId = ""; lobbyPlayerId = ""; lobbyMySeatIndex = 0; lobbyIsHost = false; lobbyPlayers = []
        try { localStorage.removeItem("bridgeActiveRoom") } catch {}
    }

    function lobbyStartGame() {
        try {
            wsClient.startGame(lobbyHiddenMode, headerState.difficulty || "Easy")
        } catch (e) { console.error("Start error:", e); alert("Failed. Is the backend running?") }
    }

    function lobbyToggleHiddenMode() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        try {
            wsClient.toggleHidden(!lobbyHiddenMode)
        } catch (e) { console.error("Toggle hidden mode error:", e) }
    }

    // ── Chat ────────────────────────────────────────────────────────
    interface ChatMsg { id: number; playerName: string; text: string; timestamp: number }
    let chatMessages = $state<ChatMsg[]>([])

    function chatSend(text: string) {
        if (!text || !roomId || !lobbyPlayerId) return
        try {
            wsClient.sendChat(lobbyPlayerId, text)
        } catch {}
    }

    // ── WS Event Listeners (replaces HTTP polling) ──────────────────
    let wsInitialized = false
    $effect(() => {
        if (!onlineToken || wsInitialized) return
        wsInitialized = true

        // Connect WebSocket
        wsClient.connect(onlineToken)
        console.log("[WS] Effect running, onlineToken present")

        const unsubCreated = wsClient.on("lobby:created", (data) => {
            console.log("[WS] lobby:created", data)
            roomId = data.roomId
            lobbyRoomId = data.roomId
            lobbyMySeatIndex = data.seatIndex
            lobbyIsHost = true
            lobbyPlayerId = data.playerId
        })

        const unsubJoined = wsClient.on("lobby:joined", (data) => {
            roomId = data.roomId
            lobbyRoomId = data.roomId
            lobbyMySeatIndex = data.seatIndex
            lobbyIsHost = false
            lobbyPlayerId = data.playerId
        })

        // Standalone game:state listener for realtime updates (handles both
        // lobby-created games and existing rooms loaded via loadExistingRoom)
        const unsubGameState = wsClient.on("game:state", (data) => {
            if (data.roomId === roomId) {
                const updated = apiStateToGame(data.state, data.roomId, data.state.betWinner ?? undefined)
                fixupPlayerDisplay(updated)
                game = updated
            }
        })

        const unsubUpdate = wsClient.on("lobby:update", (data) => {
            lobbyPlayers = data.players
            lobbyHiddenMode = data.hiddenMode
        })

        const unsubStarted = wsClient.on("lobby:started", (data) => {
            headerState.hiddenMode = lobbyHiddenMode
            hiddenModeLocked = lobbyHiddenMode
            goto(`/?room=${encodeURIComponent(data.roomId)}&seat=${lobbyMySeatIndex}`)
        })

        const unsubLeft = wsClient.on("lobby:left", () => {
            roomId = ""
            lobbyRoomId = ""
            lobbyPlayerId = ""
            lobbyMySeatIndex = 0
            lobbyIsHost = false
            lobbyPlayers = []
        })

        const unsubHiddenToggled = wsClient.on("lobby:hidden_toggled", (data) => {
            lobbyHiddenMode = data.enabled
        })

        const unsubChatMessage = wsClient.on("chat:message", (data) => {
            chatMessages = [...chatMessages, data]
        })

        // Cleanup: unsubscribe all listeners and disconnect WS on destroy
        return () => {
            unsubCreated()
            unsubJoined()
            unsubUpdate()
            unsubStarted()
            unsubLeft()
            unsubHiddenToggled()
            unsubChatMessage()
            unsubGameState()
            wsClient.disconnect()
        }
    })

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
        // Determine if the human player is on Team 1 (bet winner's team)
        const userIsTeam1 = game.Team1?.some((p: any) => p.ID === humanPlayerId) ?? false
        const wonMatch = (game.Winner === "Team 1" && userIsTeam1) || (game.Winner === "Team 2" && !userIsTeam1) ? 1 : 0
        // Partner is the bet winner's partner (in-game player ID), not the human player's teammate
        const partner = game.BetWinner?.Partner?.ID ?? 0

        const body: Record<string, unknown> = {
            date: Date.now(),
            botDifficulty: headerState.difficulty,
            trumpSuit: game.Trump,
            betSize: game.BetSize,
            betWinner: game.BetWinner?.ID ?? 0,
            partner,
            wonMatch,
            setsData: saveCompletedSetsJson,
            players: JSON.stringify((game.Players ?? []).map((p: any) => ({ id: p.ID, username: p.Username }))),
            roomId,
            betWinnerUserId: 0,
            partnerUserId: 0,
            winningTeam: userIsTeam1 ? 1 : 2,
            isHidden: headerState.hiddenMode,
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
            loadingMatchResult = true
            const res = await fetch(`${API_URL}/api/matches`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "X-Session-Token": onlineToken,
                },
                body: JSON.stringify(body),
            })
            if (res.ok) {
                const data = await res.json()
                if (data.id) lastMatchId = data.id
                if (data.eloChange !== undefined) lastEloChange = data.eloChange
            } else {
                console.error("Auto-save failed:", res.status, await res.text().catch(() => ""))
            }
            loadingMatchResult = false
        } catch (e) {
            console.error("Auto-save error:", e)
            loadingMatchResult = false
        }
    }
</script>

<div class="flex flex-col gap-6 w-full min-h-screen items-center px-4 pt-20 pb-8">

{#if page.url.searchParams.get("room")}
    {#if isOnlineLoading}
    <div class="text-lg text-muted-foreground animate-pulse">
        Starting game...
    </div>
    {:else if isOnline && game.Players}
    <div class="text-2xl text-muted-foreground">
        <p>{playerName(game.WhoseTurn)}'s turn</p>
    </div>

    {#if game.IsPartnerSelectionPhase && game.BetWinner.ID === humanPlayerId}
        <!-- Partner selection: displayed outside the game board card -->
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
            <!-- Your own hand for reference -->
            <div class="w-full max-w-3xl mt-4">
                <p class="text-xs text-muted-foreground mb-1">Your hand:</p>
                <div class="flex flex-wrap gap-1 justify-center">
                    {#each game.Players[humanSeat].Cards as card, index (index)}
                        <PokerCard card={card} isIllegal={false} minify={true} />
                    {/each}
                </div>
            </div>
        </div>
    {:else if game.IsPartnerSelectionPhase}
        <div class="flex flex-col gap-4 items-center">
            <p class="text-xl">{playerName(game.BetWinner.ID)} is selecting a partner...</p>
        </div>
    {:else}
    <!-- Play area table -->
    <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
    <div class="flex flex-col gap-4 flex-1 min-w-0 max-w-3xl">
        <!-- Game info strip -->
        <GameInfo {game} {humanSeat} {humanPlayerId} />

    <div class="rounded-xl border border-border bg-card/50 p-4 sm:p-6 relative">
        <!-- Settings gear (top-right of game board card) -->
        <div class="absolute top-2 right-2 z-10">
        <Popover.Root>
            <Popover.Trigger>
                <button class="p-1.5 rounded-md text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors">
                    <Settings class="w-4 h-4" />
                </button>
            </Popover.Trigger>
            <Popover.Content class="border w-64 mr-2 mt-1 text-sm" sideOffset={8}>
                <div class="flex flex-col gap-3 p-1">
                    <div class="flex justify-between items-center gap-4">
                        <Label for="bot-speed">Bot Speed</Label>
                        <Input type="number" bind:value={headerState.botSpeed} class="w-[100px]" />
                    </div>
                    <div class="flex justify-between items-center gap-4">
                        <Label for="hidden-mode">Hidden Mode</Label>
                        <Switch bind:checked={headerState.hiddenMode} disabled={hiddenModeLocked} />
                    </div>
                </div>
            </Popover.Content>
        </Popover.Root>
        </div>

    {#if game.IsBettingPhase}
        <BidArea {game} {humanSeat} {humanPlayerId} hiddenMode={headerState.hiddenMode} onRaise={onlineRaiseBet} onPass={onlinePassBet} />
    {:else}
        <PlayArea {game} {humanSeat} {humanPlayerId} hiddenMode={headerState.hiddenMode} onPlayCard={onlinePlayCard} />
    {/if}
    </div>
    </div>

    <!-- Chat sidebar (in-game) -->
    <Chat {roomId} lobbyPlayerId={lobbyPlayerId} bind:chatMessages onSend={chatSend} />

    </div>
    {/if}

    <!-- Game-over dialog -->
    {#if showGameOverDialog}
    <Dialog.Root bind:open={showGameOverDialog}>
        <Dialog.Content class="max-w-sm">
            <Dialog.Header class="text-center">
                <Dialog.Title
                    class="text-2xl font-bold {game.Winner === 'Team 1' && game.Team1?.some((p: any) => p.ID === humanPlayerId) || game.Winner === 'Team 2' && game.Team2?.some((p: any) => p.ID === humanPlayerId) ? 'text-[var(--blue)]' : 'text-[var(--red)]'}"
                >
                    {game.Winner === 'Team 1' && game.Team1?.some((p: any) => p.ID === humanPlayerId) || game.Winner === 'Team 2' && game.Team2?.some((p: any) => p.ID === humanPlayerId) ? 'Win!' : 'Loss'}
                </Dialog.Title>
                <Dialog.Description class="flex flex-col gap-2 items-center pt-2">
                    <span class="text-lg">Team 1 <span class="text-[var(--red)]">{team1Sets}</span> — <span class="text-[var(--blue)]">{team2Sets}</span> Team 2</span>
                    <span class="text-xs text-muted-foreground mt-1">
                        Bet: {game.BetSize}{suitToSymbol.get(game.Trump)}
                    </span>
                    {#if loadingMatchResult}
                        <span class="text-sm mt-1 text-muted-foreground animate-pulse">Calculating Elo...</span>
                    {:else if lastEloChange !== null}
                        <span class="text-sm mt-1 {lastEloChange > 0 ? 'text-green' : 'text-red'}">
                            Elo: {lastEloChange > 0 ? '+' : ''}{lastEloChange}
                        </span>
                    {/if}
                </Dialog.Description>
            </Dialog.Header>
            <Dialog.Footer class="justify-center gap-2">
                <Button onclick={() => { showGameOverDialog = false }} variant="outline">OK</Button>
                {#if loadingMatchResult}
                    <Button disabled variant="outline">Saving...</Button>
                {:else if lastMatchId}
                    <Button onclick={() => { showGameOverDialog = false; goto(`/user/${lastMatchId}`) }}>
                        View Results
                    </Button>
                {/if}
                <Button onclick={() => { showGameOverDialog = false; try { localStorage.removeItem("bridgeActiveRoom") } catch {}; goto("/") }}>
                    Play Again
                </Button>
            </Dialog.Footer>
        </Dialog.Content>
    </Dialog.Root>
    {/if}
    {/if}

{:else}
    <!-- Lobby UI -->
    <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
        <Lobby
            {onlineToken}
            {username}
            bind:lobbyRoomId
            bind:lobbyPlayerId
            bind:lobbyMySeatIndex
            bind:lobbyIsHost
            bind:lobbyPlayers
            bind:lobbyHiddenMode
            bind:difficulty={headerState.difficulty}
            oncreate={lobbyCreateRoom}
            onjoin={lobbyJoinRoom}
            onleave={lobbyLeaveRoom}
            onstart={lobbyStartGame}
            ontogglehidden={lobbyToggleHiddenMode}
        />

        {#if lobbyRoomId}
        <!-- Chat sidebar (lobby) -->
        <Chat roomId={roomId} lobbyPlayerId={lobbyPlayerId} bind:chatMessages onSend={chatSend} />
        {/if}
    </div>
{/if}

</div>
