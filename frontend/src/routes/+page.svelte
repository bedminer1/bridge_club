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
        getRoomState,
        apiStateToGame,
        frontendCardToApiCard,
    } from "$lib/game/api-game";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";

    import { wsClient } from "$lib/game/ws-client";
    import { playCardPlace, playCardDeal, playTrickWon, playGameWon, playGameLost } from "$lib/game/sound";
    import { playerName } from "$lib/game/player-utils";

    // ── Import extracted components ─────────────────────────────────
    import Lobby from "$lib/components/lobby.svelte";
    import GameInfo from "$lib/components/game-info.svelte";
    import BidArea from "$lib/components/bid-area.svelte";
    import PlayArea from "$lib/components/play-area.svelte";
    import Chat from "$lib/components/chat.svelte";
    import { EMPTY_GAME } from "$lib/game/types.js"

    let { data } = $props()
    let { username, userID, token } = $state(data)

    // Online-only game state
    let isOnline = $state(false)
    let isOnlineLoading = $state(false)
    let isJoiningRoom = $state(false)
    let game: Game = $state(EMPTY_GAME)
    let roomId = $state("")
    let onlineToken = $state(token ?? "")

    let playbackQueue: GameEvent[] = $state([])
    let isPlaybackRunning = $state(false)
    let displayedPlayCount = $state(0)
    let playbackGenerationKey: number = $state(0)
    const PLAY_DELAY_MS = 1000

    // Lock hidden mode toggle if it was on at game start
    let hiddenModeLocked = $state(false)

    // Match result data for the game-over dialog
    let lastMatchId = $state<number | null>(null)
    let lastEloChange = $state<number | null>(null)
    let loadingMatchResult = $state(false)
    let isGuest = $state(false) // true if created via guest flow
    let isDefaultPassword = $state(false) // true if guest left password empty

    // user info
    let loggedIn: boolean = $derived(userID === 0 ? false : true)

    // Fallback: if SSR session check failed but cookie exists, retry client-side
    $effect(() => {
        if (userID === 0) {
            const cookie = document.cookie.split("; ").find(r => r.startsWith("session="))
            if (cookie) {
                const tokenVal = cookie.split("=")[1]
                const api = typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')
                    ? "http://127.0.0.1:3000" : "https://bridge-club.duckdns.org"
                fetch(`${api}/api/auth/session?token=${encodeURIComponent(tokenVal)}`)
                    .then(r => r.json())
                    .then(d => {
                        if (d.ok && d.user) {
                            userID = d.user.id
                            username = d.user.username
                            onlineToken = tokenVal
                        }
                    })
                    .catch(() => {})
            }
        }
    })

    // Load a room if the URL has a ?room= param; otherwise show the lobby
    $effect(() => {
        const urlRoomId = page.url.searchParams.get("room")
        if (urlRoomId) {
            // Save active room to localStorage so we can resume after navigation
            try { localStorage.setItem("bridgeActiveRoom", JSON.stringify({ roomId: urlRoomId, seat: page.url.searchParams.get("seat") })) } catch {}
            // Save to sessionStorage too — login page reads this to redirect back
            try { sessionStorage.setItem("bridgePendingRoom", urlRoomId) } catch {}
            if (loggedIn && onlineToken && !isOnline && !isOnlineLoading && !isJoiningRoom) {
                if (page.url.searchParams.get("seat")) {
                    // Has seat param — this is a game URL, load directly
                    loadExistingRoom(urlRoomId)
                } else if (!lobbyRoomId) {
                    // No seat param and not already in a lobby — check room status
                    loadRoomOrLobby(urlRoomId)
                }
            }
        }
    })

    // Resume active room from localStorage when returning to page without ?room=
    $effect(() => {
        if (!page.url.searchParams.get("room") && loggedIn && onlineToken && !isOnline && !isOnlineLoading) {
            // If a previous room load failed, don't re-resume for this cycle
            try {
                if (sessionStorage.getItem("bridgeRoomFailed")) {
                    sessionStorage.removeItem("bridgeRoomFailed")
                    return
                }
            } catch {}
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
    let mySeatIndex = $state(0)  // Direct seat (set by lobby:started or loadExistingRoom)
    let humanSeat = $derived.by(() => {
        const s = page.url.searchParams.get("seat")
        return s !== null ? parseInt(s) : mySeatIndex
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
    const didWin = $derived(
        (game.Winner === "Team 1" && game.Team1?.some((p: any) => p.ID === humanPlayerId)) ||
        (game.Winner === "Team 2" && game.Team2?.some((p: any) => p.ID === humanPlayerId))
    )

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

    /** Load an existing room that has already started (game state). */
    async function loadExistingRoom(existingRoomId: string) {
        if (!onlineToken) return
        isOnlineLoading = true
        try {
            roomId = existingRoomId
            // Set isOnline immediately so the template switches to game view
            isOnline = true

            // Fetch room info for hidden mode setting
            let hiddenMode = true
            try {
                const infoRes = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(existingRoomId)}/info`, {
                    headers: { "X-Session-Token": onlineToken },
                })
                if (infoRes.ok) {
                    const info = await infoRes.json()
                    hiddenMode = info.hiddenMode ?? true
                }
            } catch {}
            headerState.hiddenMode = hiddenMode

            // Load the actual game state
            const gameState = await getRoomState(existingRoomId, onlineToken)
            fixupPlayerDisplay(gameState)
            game = gameState
            displayedPlayCount = extractPlayEvents(game).length
            // Play deal sound for initial load
            setTimeout(() => { for (let i = 0; i < 4; i++) setTimeout(() => playCardDeal(), i * 90) }, 100)
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

    /** Clear all saved room references (use after failed room load). */
    function clearSavedRoom() {
        try { localStorage.removeItem("bridgeActiveRoom") } catch {}
        try { sessionStorage.removeItem("bridgePendingRoom") } catch {}
        try { sessionStorage.setItem("bridgeRoomFailed", "1") } catch {}
    }

    /** Check if a room has started; load game or join lobby accordingly. */
    async function loadRoomOrLobby(roomIdToLoad: string) {
        if (!onlineToken) return
        isJoiningRoom = true
        isOnlineLoading = true
        try {
            const infoRes = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(roomIdToLoad)}/info`, {
                headers: { "X-Session-Token": onlineToken },
            })
            if (!infoRes.ok) {
                console.error("Room not found:", roomIdToLoad)
                clearSavedRoom()
                isJoiningRoom = false
                goto("/", { replaceState: true })
                return
            }
            const info = await infoRes.json()
            if (info.isStarted) {
                await loadExistingRoom(roomIdToLoad)
                isJoiningRoom = false
            } else {
                wsClient.joinLobby(roomIdToLoad)
            }
        } catch (e) {
            console.error("Failed to load room or lobby:", e)
            clearSavedRoom()
            isJoiningRoom = false
            goto("/", { replaceState: true })
        } finally {
            isOnlineLoading = false
        }
    }

    async function onlineSelectPartner(card: any) {
        if (!isOnline || !roomId || !onlineToken || isPlaybackRunning) return
        if (game.WhoseTurn !== humanPlayerId) return
        try {
            wsClient.gameAction("selectPartner", undefined, frontendCardToApiCard(card))
        } catch (e) {
            console.error("Online partner select failed:", e)
        }
    }

    /** Fix up player display: ensure IsBot is correct without overwriting real names. */
    function fixupPlayerDisplay(g: any) {
        if (!g.Players) return
        for (let i = 0; i < g.Players.length; i++) {
            g.Players[i].IsBot = i !== humanSeat
        }
    }

    // Win detection — auto-save match immediately, clear saved room
    $effect(() => {
        if (isOnline && game.Winner !== "" && loggedIn && game.Players) {
            autoSaveMatch()
            try { localStorage.removeItem("bridgeActiveRoom") } catch {}
        }
    })

    // Win detection — auto-save match immediately, clear saved room
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
            lobbyPlayers = [{ name: username, seatIndex: data.seatIndex, isBot: false }]
        })

        const unsubJoined = wsClient.on("lobby:joined", (data) => {
            roomId = data.roomId
            lobbyRoomId = data.roomId
            lobbyMySeatIndex = data.seatIndex
            lobbyIsHost = false
            lobbyPlayerId = data.playerId
            lobbyPlayers = [{ name: username, seatIndex: data.seatIndex, isBot: false }]
            isJoiningRoom = false
        })

        // Standalone game:state listener for realtime updates (handles both
        // lobby-created games and existing rooms loaded via loadExistingRoom)
        // Only process when in game mode (has ?seat= param), not during lobby
        const unsubGameState = wsClient.on("game:state", (data) => {
            if (data.roomId === roomId && isOnline) {
                const updatedGame = apiStateToGame(data.state, data.roomId, data.state.betWinner ?? undefined)
                console.log(updatedGame)
                fixupPlayerDisplay(updatedGame)
                renderGame(updatedGame)
            }
        })

        const unsubUpdate = wsClient.on("lobby:update", (data) => {
            lobbyPlayers = data.players
            lobbyHiddenMode = data.hiddenMode
        })

        const unsubStarted = wsClient.on("lobby:started", (data) => {
            headerState.hiddenMode = lobbyHiddenMode
            hiddenModeLocked = lobbyHiddenMode
            // Set human seat directly (don't rely on URL param that may not persist)
            mySeatIndex = lobbyMySeatIndex
            // Play deal sound as the game starts
            for (let i = 0; i < 4; i++) setTimeout(() => playCardDeal(), i * 90)
            // Directly load the game state without navigation
            // (goto would remount the component and lose all state)
            loadExistingRoom(data.roomId)
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


    // ── Renders the game one play at a time ──────────────────

    function sameVisualPhase(a: Game, b: Game): boolean {
        return a.IsBettingPhase === b.IsBettingPhase
            && a.IsPartnerSelectionPhase === b.IsPartnerSelectionPhase
    }
    
    // diffs updatedGame with game
    function diffGameState(updatedGame: Game, game: Game): GameEvent[] {
        const updatedGameEvents = extractPlayEvents(updatedGame)
        const gameEvents = [...extractPlayEvents(game), ...playbackQueue]
        if (gameEvents.length > updatedGameEvents.length) {
            console.log(gameEvents)
            console.log(updatedGameEvents)
            throw Error("gameEvents is longer than updatedGameEvents")
        }
        for (let i = 0; i < gameEvents.length; i++) {
            if (gameEvents[i]?.id !== updatedGameEvents[i]?.id) throw Error("gameEvents does not match updatedGameEvents")
        }
        const diff = updatedGameEvents.slice(gameEvents.length)
        console.log(diff)
        return diff
    }

    // takes any game state and returns an array of plays made in the game so far
    function extractPlayEvents(game: Game | null): GameEvent[] {
        if (!game) return []
        const events: GameEvent[] = []
        const completedSets = game.CompletedSets ?? []
        if (completedSets.length) {
            for (let i = 0; i < completedSets.length; i++){
                const set = completedSets[i]
                for (let j = 0; j < set.Cards.length; j++) {
                    const playerId = set.PlayerIDs[j]
                    const card = set.Cards[j]
                    events.push({
                        kind: "play",
                        id: `${i}-${j}-${playerId}-${card.Suit}${card.Value}`,
                        trickIndex: i,
                        position: j,
                        playerId,
                        card,
                        isTrickEnd: j === set.Cards.length - 1,
                        trickWinnerId: j === set.Cards.length - 1 ? set.WinnerID : null,
                    })
                }
            }
        }

        const curr = completedSets.length
        const currentMoves = game.Moves ?? []
        for (let i = 0; i < currentMoves.length; i++) {
            const m = currentMoves[i]
            events.push({
                kind: "play",
                id: `${curr}-${i}-${m.PlayerID}-${m.CardPlayed.Suit}${m.CardPlayed.Value}`,
                trickIndex: curr,
                position: i,
                playerId: m.PlayerID,
                card: m.CardPlayed,
                isTrickEnd: i === 3,
                trickWinnerId: null,
            })
        }

        // If the game has a winner, append a terminal 'win' event so playback
        // can render the final outcome in sequence with the plays.
        if (game && game.Winner) {
            events.push({ kind: "win", id: `win-${game.Winner}-${(game.CompletedSets ?? []).length}`, winner: game.Winner })
        }

        return events
    }

    // renders one GameEvent 
    function applyPlayEventToGame(gameEvent: GameEvent): void {
        if (gameEvent.kind === "win") {
            game.Winner = gameEvent.winner
            // Play game over sound — check if human's team won
            const humanOnTeam1 = game.Team1?.some((p: any) => p.ID === humanPlayerId) ?? false
            const humanOnTeam2 = game.Team2?.some((p: any) => p.ID === humanPlayerId) ?? false
            const humanWon = (humanOnTeam1 && gameEvent.winner === "Team 1") || (humanOnTeam2 && gameEvent.winner === "Team 2")
            if (humanWon) { playGameWon() } else { playGameLost() }
            displayedPlayCount += 1
            return
        }

        const playerIndex = game.Players.findIndex((player) => player.ID === gameEvent.playerId)
        if (playerIndex === -1) {
            console.warn("[playback] Missing player for event", gameEvent)
            throw Error("missing player for play event")
        }

        if (game.IsBettingPhase) {
            const cardToPlay = { ...gameEvent.card, WonSet: false }
            game.Moves = [
                ...(game.Moves ?? []),
                { CardPlayed: cardToPlay, PlayerID: gameEvent.playerId },
            ]
            game.WhoseTurn = (gameEvent.playerId % 4) + 1
            displayedPlayCount += 1
            return
        }

        const player = game.Players[playerIndex]
        const cardToPlay = { ...gameEvent.card, WonSet: false }
        const handIndex = player.Cards.findIndex((card) => card.Suit === cardToPlay.Suit && card.Value === cardToPlay.Value)
        if (handIndex !== -1) {
            player.Cards.splice(handIndex, 1)
        }
        player.PlayedCards.push(cardToPlay)

        const currentMoves: Move[] = [...(game.Moves ?? []), { CardPlayed: cardToPlay, PlayerID: gameEvent.playerId }]
        game.TrumpPlayed = game.TrumpPlayed || cardToPlay.Suit === game.Trump

        if (currentMoves.length < 4) {
            game.Moves = currentMoves
            if (currentMoves.length === 1) {
                game.TurnSuit = cardToPlay.Suit
            }
            game.WhoseTurn = (gameEvent.playerId % 4) + 1
            playCardPlace()
        } else {
            const winnerId = gameEvent.trickWinnerId ?? gameEvent.playerId
            const completedCards = currentMoves.map((move) => ({
                ...move.CardPlayed,
                WonSet: move.PlayerID === winnerId,
            }))

            game.PreviousMoves = currentMoves.map((move) => ({
                CardPlayed: {
                    ...move.CardPlayed,
                    WonSet: move.PlayerID === winnerId,
                },
                PlayerID: move.PlayerID,
            }))
            game.CompletedSets = [
                ...(game.CompletedSets ?? []),
                {
                    Cards: completedCards,
                    WinnerID: winnerId,
                    PlayerIDs: currentMoves.map((move) => move.PlayerID),
                },
            ]
            game.Moves = []
            game.TurnSuit = ""

            const winningPlayerIndex = game.Players.findIndex((player) => player.ID === winnerId)
            if (winningPlayerIndex !== -1) {
                game.Players[winningPlayerIndex].Sets += 1
            }
            game.WhoseTurn = winnerId
            playTrickWon()
        }

        displayedPlayCount += 1
    }

    // renders all GameEvents in the playbackQueue one by one
    async function processPlaybackQueue(): Promise<void> {
        if (isPlaybackRunning) return
        isPlaybackRunning = true
        const key = ++playbackGenerationKey
        while (playbackQueue.length > 0 && key === playbackGenerationKey) {
            const playEvent = playbackQueue.shift()!
            applyPlayEventToGame(playEvent)
            await new Promise(r => setTimeout(r, PLAY_DELAY_MS))
        }
        isPlaybackRunning = false
    }

    // orchestrator for all the helper functions
    function renderGame(updatedGame: Game): void {
        try {
            if (!sameVisualPhase(updatedGame, game)) {
                game = updatedGame
                playbackQueue = []
                isPlaybackRunning = false
                playbackGenerationKey += 1
                displayedPlayCount = extractPlayEvents(game).length
                return
            }
            const diff = diffGameState(updatedGame, game)
            playbackQueue = [...playbackQueue, ...diff]
            console.log($state.snapshot(playbackQueue))
            processPlaybackQueue()
        } catch (Error) {
            console.log(Error)
            console.log("frontend game state is malformed. falling back to latest backend game state.")
            console.log($state.snapshot(playbackQueue))
            game = updatedGame
            playbackQueue = []
            isPlaybackRunning = false
            playbackGenerationKey += 1
            displayedPlayCount = extractPlayEvents(game).length
        }
    }

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
        const partnerId = game.BetWinner?.Partner?.ID ?? null

        // Build participants array
        const participants = (game.Players ?? []).map((p: any, i: number) => ({
            username: p.Username,
            seatIndex: i,
            team: (userIsTeam1 && (p.ID === game.BetWinner?.ID || p.ID === partnerId)) || (!userIsTeam1 && p.ID !== game.BetWinner?.ID && p.ID !== partnerId) ? 1 : 2,
            setsWon: p.Sets ?? 0,
            cardsPlayed: JSON.stringify(savePlayedCards[i] ?? []),
        }))

        // Fix teams: Team 1 = bet winner + partner, Team 2 = everyone else
        const betWinnerSeat = game.Players?.findIndex((p: any) => p.ID === game.BetWinner?.ID) ?? 0
        const partnerSeat = game.Players?.findIndex((p: any) => p.ID === partnerId) ?? -1
        for (const p of participants) {
            p.team = (p.seatIndex === betWinnerSeat || p.seatIndex === partnerSeat) ? 1 : 2
        }

        // Compute team sets
        const team1Sets = participants.filter((p: any) => p.team === 1).reduce((s: number, p: any) => s + p.setsWon, 0)
        const team2Sets = participants.filter((p: any) => p.team === 2).reduce((s: number, p: any) => s + p.setsWon, 0)
        const target = 6 + game.BetSize
        const team1Won = team1Sets >= target

        const body: Record<string, unknown> = {
            roomId,
            createdAt: Date.now(),
            trumpSuit: game.Trump,
            betSize: game.BetSize,
            betWinnerIdx: betWinnerSeat,
            partnerIdx: partnerSeat >= 0 ? partnerSeat : null,
            partnerCard: game.PartnerCard?.Rank ? JSON.stringify(game.PartnerCard) : null,
            winningTeam: team1Won ? 1 : 2,
            setsData: saveCompletedSetsJson,
            matchType: "single",
            isHidden: headerState.hiddenMode,
            participants,
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

<div class="flex flex-col gap-6 w-full items-center px-4 pt-20 pb-8">

{#if isOnline || page.url.searchParams.get("room")}
    {#if isOnlineLoading}
    <div class="text-lg text-muted-foreground animate-pulse">
        Starting game...
    </div>
    {:else if isOnline && game.Players}
    <div class="text-2xl text-muted-foreground">
        <p>{playerName(game, game.WhoseTurn)}'s turn</p>
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
            <p class="text-xl">{playerName(game, game.BetWinner.ID)} is selecting a partner...</p>
        </div>
    {:else}
    <!-- Play area table -->
    <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
    <div class="flex flex-col gap-4 flex-1 min-w-0 max-w-3xl {isPlaybackRunning ? 'pointer-events-none opacity-60' : ''}" aria-busy={isPlaybackRunning}>
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
        <BidArea {game} {humanSeat} {humanPlayerId} hiddenMode={headerState.hiddenMode} disabled={isPlaybackRunning} {roomId} />
    {:else}
        <PlayArea {game} {humanSeat} {humanPlayerId} hiddenMode={headerState.hiddenMode} disabled={isPlaybackRunning} {roomId} />
    {/if}
    </div>
    </div>

    <!-- Chat sidebar (in-game) -->
    <Chat {roomId} lobbyPlayerId={lobbyPlayerId} bind:chatMessages onSend={chatSend} />

    </div>
    {/if}
    {:else}
    <!-- Lobby UI (room hasn't started yet) -->
    <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
        <Lobby
            {onlineToken}
            {username}
            {userID}
            bind:lobbyRoomId
            bind:lobbyPlayerId
            bind:lobbyMySeatIndex
            bind:lobbyIsHost
            bind:lobbyPlayers
            bind:lobbyHiddenMode
            bind:difficulty={headerState.difficulty}
            onguestlogin={(name: string, token: string, uid: number, defaultPw: boolean) => {
                username = name; onlineToken = token; userID = uid; isGuest = true; isDefaultPassword = defaultPw;
                const urlRoomId = page.url.searchParams.get("room")
                if (urlRoomId && !isOnline) { loadRoomOrLobby(urlRoomId) }
            }}
        />

        {#if lobbyRoomId}
        <Chat roomId={roomId} lobbyPlayerId={lobbyPlayerId} bind:chatMessages onSend={chatSend} />
        {/if}
    </div>
    {/if}

    <!-- Game-over dialog -->
    {#if showGameOverDialog}
    <Dialog.Root bind:open={showGameOverDialog}>
        <Dialog.Content class="max-w-sm">
            <Dialog.Header class="text-center">
                <Dialog.Title class="text-3xl font-bold {didWin ? 'text-[var(--blue)]' : 'text-[var(--red)]'}">
                    {didWin ? 'You Won!' : 'Loss'}
                </Dialog.Title>
                <Dialog.Description class="flex flex-col gap-3 items-center pt-3">
                    <!-- Team scores -->
                    <div class="flex items-center gap-4 text-lg">
                        <div class="flex flex-col items-center gap-0.5">
                            <span class="text-xs text-muted-foreground uppercase tracking-wide">Team 1</span>
                            <span class="text-2xl font-bold tabular-nums">{team1Sets}</span>
                        </div>
                        <span class="text-xl text-muted-foreground/40 font-light">:</span>
                        <div class="flex flex-col items-center gap-0.5">
                            <span class="text-xs text-muted-foreground uppercase tracking-wide">Team 2</span>
                            <span class="text-2xl font-bold tabular-nums">{team2Sets}</span>
                        </div>
                    </div>
                    <!-- Bet info -->
                    <div class="flex items-center gap-3 text-xs text-muted-foreground">
                        <span class="rounded border border-border px-2 py-0.5">{game.BetSize}{suitToSymbol.get(game.Trump)}</span>
                        <span>Target: {6 + game.BetSize}</span>
                    </div>
                    <!-- Per-player sets -->
                    <div class="flex flex-wrap justify-center gap-x-4 gap-y-1 text-xs">
                        {#each game.Players ?? [] as player, i}
                            <span style="color: {['var(--red)', 'var(--blue)', 'var(--yellow)', 'var(--green)'][i]}">
                                {player.Username}: {player.Sets ?? 0} sets
                            </span>
                        {/each}
                    </div>
                    <!-- Loading / Elo -->
                    {#if loadingMatchResult}
                        <span class="text-sm mt-1 text-muted-foreground animate-pulse">Saving match...</span>
                    {:else if lastEloChange !== null}
                        <span class="text-sm font-bold {lastEloChange > 0 ? 'text-green' : 'text-red'}">
                            Elo {lastEloChange > 0 ? '+' : ''}{lastEloChange}
                        </span>
                    {/if}
                </Dialog.Description>
            </Dialog.Header>
            <Dialog.Footer class="justify-center gap-2 flex-wrap">
                <Button onclick={() => { showGameOverDialog = false }} variant="outline" size="sm">Close</Button>
                {#if loadingMatchResult}
                    <Button disabled variant="outline" size="sm">Saving...</Button>
                {:else if lastMatchId}
                    <Button onclick={() => { showGameOverDialog = false; goto(`/user/${lastMatchId}`) }} size="sm">
                        View Results
                    </Button>
                {/if}
                <Button onclick={() => { showGameOverDialog = false; try { localStorage.removeItem("bridgeActiveRoom") } catch {}; goto("/") }} size="sm">
                    Play Again
                </Button>
            </Dialog.Footer>
            <!-- Password change prompt for guest users -->
            {#if isGuest && isDefaultPassword && !loadingMatchResult && lastMatchId}
                <div class="px-6 pb-4 pt-1">
                    <div class="flex items-center gap-2 rounded-lg border border-border bg-muted/30 p-3 text-xs text-muted-foreground">
                        <span>🔑</span>
                        <span>Update your password in</span>
                        <a href="/user" class="text-accent hover:underline ml-auto shrink-0">Settings</a>
                    </div>
                </div>
            {/if}
        </Dialog.Content>
    </Dialog.Root>
    {/if}

{:else}
    <!-- Lobby UI -->
    <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
        <Lobby
            {onlineToken}
            {username}
            {userID}
            bind:lobbyRoomId
            bind:lobbyPlayerId
            bind:lobbyMySeatIndex
            bind:lobbyIsHost
            bind:lobbyPlayers
            bind:lobbyHiddenMode
            bind:difficulty={headerState.difficulty}
            onguestlogin={(name: string, token: string, uid: number, defaultPw: boolean) => {
                username = name; onlineToken = token; userID = uid; isGuest = true; isDefaultPassword = defaultPw;
                // Auto-join room if there's a pending invite in the URL
                const urlRoomId = page.url.searchParams.get("room")
                if (urlRoomId && !isOnline) { loadExistingRoom(urlRoomId) }
            }}
        />

        {#if lobbyRoomId}
        <!-- Chat sidebar (lobby) -->
        <Chat roomId={roomId} lobbyPlayerId={lobbyPlayerId} bind:chatMessages onSend={chatSend} />
        {/if}
    </div>
{/if}

</div>
