<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
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
    import { goto } from "$app/navigation";

    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card/index.js";

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
            // Always start chat polling when entering the game
            chatStartPolling()
            // Start game polling if it's not the human's turn
            if (game.WhoseTurn !== humanPlayerId && game.Winner === "") {
                startPolling()
            }
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

    /** Poll timeout handle for online bot turn waiting. */
    let pollTimeout: ReturnType<typeof setTimeout> | null = null

    function startPolling() {
        stopPolling()
        _pollActive = true
        chatStartPolling()
        const delay = (headerState.botSpeed ?? 2) * 1000
        pollTimeout = setTimeout(() => {
            if (_pollActive) tick()
        }, delay)
    }

    let _pollActive = $state(false)

    function stopPolling() {
        chatStopPolling()
        _pollActive = false
        if (pollTimeout !== null) { clearTimeout(pollTimeout); pollTimeout = null }
    }

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

    /** Fix up player display: ensure IsBot is correct without overwriting real names. */
    function fixupPlayerDisplay(g: any) {
        if (!g.Players) return
        for (let i = 0; i < g.Players.length; i++) {
            g.Players[i].IsBot = i !== humanSeat
        }
    }

    // Cleanup polling on component destroy
    $effect(() => {
        return () => stopPolling()
    })

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
    let lobbyMode = $state<"" | "create" | "join">("create")
    let lobbyCreating = $state(false)
    let lobbyIsHost = $state(false)
    let lobbyRoomId = $state("")
    let lobbyPlayerId = $state("")
    let lobbyJoinRoomId = $state("")
    let lobbyJoining = $state(false)
    let lobbyJoinError = $state("")
    let lobbyMySeatIndex = $state(0)
    let lobbyPlayers = $state<Array<{ name: string; seatIndex: number; isBot: boolean }>>([])
    let lobbyHiddenMode = $state(true)
    let lobbyPollInterval: ReturnType<typeof setInterval> | null = null

    async function lobbyCreateRoom() {
        lobbyCreating = true
        try {
            const res = await fetch(`${API_URL}/api/rooms`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": onlineToken },
            })
            if (!res.ok) { const t = await res.text().catch(() => ""); alert(`Failed: ${res.status} ${t}`); return }
            const d = await res.json()
            roomId = d.roomId; lobbyRoomId = d.roomId; lobbyMySeatIndex = d.seatIndex; lobbyIsHost = true; lobbyPlayerId = d.playerId
            lobbyStartPolling(); chatStartPolling()
        } catch (e) { console.error("Create room error:", e); alert("Failed. Is the backend running?") }
        finally { lobbyCreating = false }
    }

    async function lobbyJoinRoom() {
        if (!lobbyJoinRoomId.trim()) return
        lobbyJoining = true; lobbyJoinError = ""
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(lobbyJoinRoomId.trim())}/join`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": onlineToken },
            })
            if (!res.ok) { const t = await res.text().catch(() => ""); lobbyJoinError = `Failed: ${res.status} ${t}`; return }
            const d = await res.json()
            roomId = d.roomId; lobbyRoomId = d.roomId; lobbyMySeatIndex = d.seatIndex; lobbyIsHost = false; lobbyPlayerId = d.playerId
            lobbyStartPolling(); chatStartPolling()
        } catch (e) { console.error("Join error:", e); lobbyJoinError = "Failed. Is the backend running?" }
        finally { lobbyJoining = false }
    }

    async function lobbyLeaveRoom() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        try {
            await fetch(`${API_URL}/api/rooms/${encodeURIComponent(lobbyRoomId)}/leave/${encodeURIComponent(lobbyPlayerId)}`, {
                method: "POST",
                headers: { "X-Session-Token": onlineToken },
            })
        } catch (e) { console.error("Leave error:", e) }
        chatStopPolling(); lobbyStopPolling()
        roomId = ""; lobbyRoomId = ""; lobbyPlayerId = ""; lobbyMySeatIndex = 0; lobbyIsHost = false; lobbyPlayers = []
        try { localStorage.removeItem("bridgeActiveRoom") } catch {}
    }

    async function lobbyStartGame() {
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(lobbyRoomId)}/start`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": onlineToken },
            })
            if (!res.ok) { const t = await res.text().catch(() => ""); alert(`Start failed: ${res.status} ${t}`); return }
            const d = await res.json()
            if (d.ok) { lobbyStopPolling(); headerState.hiddenMode = lobbyHiddenMode; goto(`/?room=${encodeURIComponent(lobbyRoomId)}&seat=${lobbyMySeatIndex}`) }
            else { alert("Start failed: " + JSON.stringify(d)) }
        } catch (e) { console.error("Start error:", e); alert("Failed. Is the backend running?") }
    }

    function lobbyStartPolling() {
        lobbyStopPolling(); lobbyPoll()
        lobbyPollInterval = setInterval(lobbyPoll, 2000)
    }
    function lobbyStopPolling() {
        if (lobbyPollInterval !== null) { clearInterval(lobbyPollInterval); lobbyPollInterval = null }
    }
    async function lobbyPoll() {
        if (!lobbyRoomId) return
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(lobbyRoomId)}/info`, {
                headers: { "X-Session-Token": onlineToken },
            })
            if (!res.ok) return
            const d = await res.json()
            if (d.isStarted) { lobbyStopPolling(); headerState.hiddenMode = d.hiddenMode ?? true; goto(`/?room=${encodeURIComponent(lobbyRoomId)}&seat=${lobbyMySeatIndex}`); return }
            lobbyPlayers = d.players || []
            lobbyHiddenMode = d.hiddenMode ?? true
        } catch (e) { console.error("Poll error:", e) }
    }

    async function lobbyToggleHiddenMode() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        const newVal = !lobbyHiddenMode
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(lobbyRoomId)}/hidden-mode`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": onlineToken },
                body: JSON.stringify({ enabled: newVal, playerId: lobbyPlayerId }),
            })
            if (res.ok) lobbyHiddenMode = newVal
        } catch (e) { console.error("Toggle hidden mode error:", e) }
    }
    async function lobbyCopyRoomId() { try { await navigator.clipboard.writeText(lobbyRoomId) } catch {} }
    $effect(() => { return () => lobbyStopPolling() })

    // ── Chat ────────────────────────────────────────────────────────
    interface ChatMsg { id: number; playerName: string; text: string; timestamp: number }
    let chatMessages = $state<ChatMsg[]>([])
    function fmtChatTime(ts: number): string {
        return new Date(ts).toLocaleTimeString("en-SG", { hour: "2-digit", minute: "2-digit", hour12: false })
    }
    let chatText = $state("")
    let chatLastId = $state(0)
    let chatPollInterval: ReturnType<typeof setInterval> | null = null
    let chatContainer: HTMLDivElement | undefined = $state(undefined)

    function chatStartPolling() {
        chatStopPolling(); chatPoll()
        chatPollInterval = setInterval(chatPoll, 2000)
    }
    function chatStopPolling() {
        if (chatPollInterval !== null) { clearInterval(chatPollInterval); chatPollInterval = null }
    }
    async function chatPoll() {
        if (!roomId) return
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(roomId)}/chat?after=${chatLastId}`, {
                headers: { "X-Session-Token": onlineToken },
            })
            if (!res.ok) return
            const d = await res.json()
            if (d.messages && d.messages.length > 0) {
                for (const m of d.messages) {
                    chatMessages = [...chatMessages, m]
                    if (m.id > chatLastId) chatLastId = m.id
                }
                // Scroll to bottom
                if (chatContainer) setTimeout(() => { chatContainer.scrollTop = chatContainer.scrollHeight }, 50)
            }
        } catch {}
    }
    async function chatSend() {
        const text = chatText.trim()
        if (!text || !roomId || !lobbyPlayerId) return
        chatText = ""
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(roomId)}/chat`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": onlineToken },
                body: JSON.stringify({ playerId: lobbyPlayerId, text }),
            })
            if (res.ok) {
                const d = await res.json()
                if (d.ok && d.message) {
                    chatMessages = [...chatMessages, d.message]
                    chatLastId = d.message.id
                    if (chatContainer) setTimeout(() => { chatContainer.scrollTop = chatContainer.scrollHeight }, 50)
                }
            }
        } catch {}
    }
    function chatHandleKey(e: KeyboardEvent) {
        if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); chatSend() }
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
        // Use ID comparison, not reference equality (Team1/Team2 arrays are always different instances)
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
            // Required by backend SaveMatchRequest (computed server-side from seat data)
            betWinnerUserId: 0,
            partnerUserId: 0,
            winningTeam: userIsTeam1 ? 1 : 2,
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
            const res = await fetch(`${API_URL}/api/matches`, {
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
    {:else if game.IsPartnerSelectionPhase}
        <div class="flex flex-col gap-4 items-center">
            <p class="text-xl">{playerName(game.BetWinner.ID)} is selecting a partner...</p>
        </div>
    {:else}
    <!-- Play area table -->
    <div class="flex gap-4 w-full justify-center">
    <div class="flex flex-col gap-4 flex-1 min-w-0 max-w-3xl">
        <!-- Game info strip -->
        <div class="flex flex-nowrap gap-x-2 sm:gap-x-4 text-2xs sm:text-xs text-muted-foreground px-1 overflow-x-auto scrollbar-none">
            {#if game.BetSize > 0 || !game.IsBettingPhase}
            <span>Trump <strong class="text-accent font-medium">{suitToSymbol.get(game.Trump)} {game.Trump}</strong></span>
            <span>Bet <strong class="text-foreground font-medium">{game.BetSize}</strong></span>
            {/if}
            {#if game.IsBettingPhase && game.BetSize > 0}
            <span class="text-muted-foreground/40">|</span>
            <span class="whitespace-nowrap"><strong class="text-foreground font-medium">{playerName(game.BetWinner.ID)}</strong> + partner need <strong class="text-accent font-medium">{6 + game.BetSize}</strong> sets</span>
            <span class="whitespace-nowrap">Opponents need <strong class="text-foreground font-medium">{8 - game.BetSize}</strong> sets</span>
            {/if}
            {#if !game.IsBettingPhase}
            <span>Winner <strong class="text-foreground font-medium">{playerName(game.BetWinner.ID)}</strong></span>
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
                <span class="text-{playerIDToColor.get(move.PlayerID)}">{playerName(move.PlayerID)}</span>
                {#if move.CardPlayed.Value === 0}
                    <span>passed</span>
                {:else}
                    <span>raised <strong>{move.CardPlayed.Value} {move.CardPlayed.Suit}</strong></span>
                {/if}
            </div>
        {/each}
        {#if game.Moves.length < 4}
            <div class="text-xs text-muted-foreground/40 mt-1">
                waiting for {playerName(game.WhoseTurn)}...
            </div>
        {/if}
        {#if game.Moves.length === 0}
            <div class="text-xs text-muted-foreground/40">
                {playerName(game.WhoseTurn)} to bet
            </div>
        {/if}
    </div>
    {:else}
    <div class="flex justify-between w-full min-h-28 relative">
        <div class="flex gap-1.5 sm:gap-2 ml-2 sm:ml-4 items-start flex-shrink-0">
            {#each game.Moves as move, i}
            <div class="flex flex-col {i % 2 === 0 ? 'items-center mb-8' : 'items-center mt-8'}">
                <PokerCard card={move.CardPlayed} isIllegal={false} minify={false} />
                <p class="text-{playerIDToColor.get(move.PlayerID)} text-2xs sm:text-xs whitespace-nowrap">{playerName(move.PlayerID)}</p>
            </div>
            {/each}
        </div>
    
        {#if game.PreviousMoves.length !== 0} 
        <div class="relative mr-7 flex-shrink-0" style="width: {game.PreviousMoves.length * 12 + 40}px; min-height: {game.PreviousMoves.length * 8 + 40}px;">
            {#each game.PreviousMoves as move, i}
                <div class="absolute flex-col items-center gap-4" style="top: {i * 16}px; left: {i * 16}px; z-index: {i};">
                    <p class="text-2xs text-right text-{playerIDToColor.get(move.PlayerID)} whitespace-nowrap">P{move.PlayerID}</p>
                    <div>
                        <PokerCard card={move.CardPlayed} isIllegal={false} minify={true} />
                    </div>
                </div>
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
                <p>| Partner is {playerName(player.Partner?.ID ?? 0)}</p>
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
                <p class="text-{playerIDToColor.get(player.ID)}">{player.Username} ({player.Sets} sets) </p>
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

    <!-- Chat card (in-game, side panel) -->
    <div class="w-80 flex flex-col mt-4">
        <Card class="h-full flex flex-col">
            <CardContent class="flex flex-col gap-2 p-3 flex-1">
                <div class="text-xs font-medium text-muted-foreground">Chat</div>
                <div bind:this={chatContainer} class="flex-1 overflow-y-auto space-y-1 scrollbar-thin">
                    {#each chatMessages as msg, i (msg.id)}
                        <div>
                            {#if i === 0 || chatMessages[i-1].playerName !== msg.playerName}
                                <span class="text-xs font-semibold text-accent">{msg.playerName}</span>
                                <span class="text-xs text-muted-foreground tabular-nums">{fmtChatTime(msg.timestamp)}</span>
                            {/if}
                            <div class="text-sm text-foreground/90 break-words">{msg.text}</div>
                        </div>
                    {/each}
                </div>
                <div class="flex gap-2">
                    <Input
                        bind:value={chatText}
                        onkeydown={chatHandleKey}
                        placeholder="Chat..."
                        maxlength={500}
                        class="flex-1 h-8 text-xs"
                    />
                    <Button onclick={chatSend} size="sm" class="h-8 px-3 text-xs">Send</Button>
                </div>
            </CardContent>
        </Card>
    </div>

    </div>
    <!-- ^ closes the outer flex wrapper (play area + chat) -->
    {/if}

    <!-- Game-over dialog -->
    {#if showGameOverDialog}
    <Dialog.Root bind:open={showGameOverDialog}>
        <Dialog.Content class="max-w-sm">
            <Dialog.Header class="text-center">
                <Dialog.Title
                    class="text-2xl font-bold {game.Winner === 'Team 1' && game.Team1?.some((p: any) => p.ID === humanPlayerId) || game.Winner === 'Team 2' && game.Team2?.some((p: any) => p.ID === humanPlayerId) ? 'text-[var(--green)]' : 'text-[var(--red)]'}"
                >
                    {game.Winner === 'Team 1' && game.Team1?.some((p: any) => p.ID === humanPlayerId) || game.Winner === 'Team 2' && game.Team2?.some((p: any) => p.ID === humanPlayerId) ? 'Victory!' : 'Defeat'}
                </Dialog.Title>
                <Dialog.Description class="flex flex-col gap-2 items-center pt-2">
                    <span class="text-lg">Team 1 <span class="text-[var(--red)]">{team1Sets}</span> — <span class="text-[var(--blue)]">{team2Sets}</span> Team 2</span>
                    <span class="text-xs text-muted-foreground mt-1">
                        Bet: {game.BetSize}{suitToSymbol.get(game.Trump)}
                    </span>
                </Dialog.Description>
            </Dialog.Header>
            <Dialog.Footer class="justify-center">
                <Button onclick={() => { showGameOverDialog = false }}>OK</Button>
            </Dialog.Footer>
        </Dialog.Content>
    </Dialog.Root>
    {/if}
    {/if}

    {:else}
    <!-- Lobby UI -->
    <div class="flex gap-4 w-full justify-center">
        <div class="w-full max-w-md">
            {#if !lobbyRoomId}
                <!-- Mode Selector (always visible) -->
                <div class="relative flex justify-center mb-6 items-center">
                    {#if lobbyMode}
                    <button onclick={() => lobbyMode = ""} class="absolute left-0 p-1.5 rounded-md text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors text-4xl leading-none" title="Back">
                        ←
                    </button>
                    {/if}
                    <div class="flex gap-2">
                        <Button onclick={() => { if (lobbyMode === "create") lobbyCreateRoom(); else lobbyMode = "create" }}
                            variant={lobbyMode === "create" ? "default" : "outline"}>
                            Create Room
                        </Button>
                        <Button onclick={() => { lobbyMode = "join" }}
                            variant={lobbyMode === "join" ? "default" : "outline"}>
                            Join Room
                        </Button>
                    </div>
                </div>

                {#if lobbyMode === "create"}
                    <Card>
                        <CardHeader>
                            <CardTitle class="text-center">Create a Room</CardTitle>
                            <CardDescription class="text-center max-w-[220px] mx-auto">Create a new game room</CardDescription>
                        </CardHeader>
                        <CardContent class="flex flex-col gap-4">
                            <Button onclick={lobbyCreateRoom} disabled={lobbyCreating}>
                                {lobbyCreating ? "Creating..." : "Create Room"}
                            </Button>
                        </CardContent>
                    </Card>
                {:else if lobbyMode === "join"}
                    <Card>
                        <CardHeader>
                            <CardTitle class="text-center">Join a Room</CardTitle>
                            <CardDescription class="text-center max-w-[220px] mx-auto">Paste the room ID from the host</CardDescription>
                        </CardHeader>
                        <CardContent class="flex flex-col gap-4">
                            <div class="flex flex-col gap-2">
                                <label for="room-id" class="text-sm text-muted-foreground">Room ID</label>
                                <Input id="room-id" bind:value={lobbyJoinRoomId} placeholder="Enter room ID" />
                            </div>
                            {#if lobbyJoinError}<p class="text-sm text-destructive">{lobbyJoinError}</p>{/if}
                            <Button onclick={lobbyJoinRoom} disabled={lobbyJoining || !lobbyJoinRoomId.trim()}>
                                {lobbyJoining ? "Joining..." : "Join Room"}
                            </Button>
                        </CardContent>
                    </Card>
                {/if}
            {:else}
                <!-- Lobby View (waiting room) -->
                <Card>
                    <CardHeader class="relative">
                        <button onclick={lobbyLeaveRoom} class="absolute left-3 inset-y-0 flex items-center text-3xl leading-none text-muted-foreground hover:text-foreground transition-colors" title="Leave room">
                            ←
                        </button>
                        <CardTitle class="text-center">Game Lobby</CardTitle>
                        <CardDescription class="text-center max-w-[220px] mx-auto">Copy Room ID and invite friends</CardDescription>
                    </CardHeader>
                    <CardContent class="flex flex-col gap-4">
                        <div class="flex items-center gap-2 p-3 rounded-lg border border-border bg-muted/50">
                            <span class="text-sm font-mono text-muted-foreground flex-1 truncate">{lobbyRoomId}</span>
                            <Button onclick={lobbyCopyRoomId} variant="outline" size="sm">Copy</Button>
                        </div>
                        <div class="flex flex-col gap-2">
                            <h3 class="text-sm font-medium text-foreground">Players ({lobbyPlayers.length})</h3>
                            {#if lobbyPlayers.length === 0}
                                <p class="text-sm text-muted-foreground">Waiting for players...</p>
                            {:else}
                                <div class="flex flex-col gap-1">
                                    {#each lobbyPlayers as p}
                                        <div class="flex items-center gap-2 px-3 py-2 rounded-md border border-border bg-card">
                                            <div class="w-2 h-2 rounded-full {p.isBot ? 'bg-muted-foreground/40' : 'bg-green-500'}"></div>
                                            <span class="text-sm text-foreground">{p.name}</span>
                                            {#if p.isBot}<span class="text-xs text-muted-foreground">(bot)</span>{/if}
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                        {#if lobbyIsHost}
                            <div class="flex items-center justify-between px-3 py-2 rounded-md border border-border bg-card">
                                <span class="text-sm text-foreground">Hidden Mode Only</span>
                                <button
                                    onclick={lobbyToggleHiddenMode}
                                    class="relative w-10 h-5 rounded-full transition-colors {lobbyHiddenMode ? 'bg-accent' : 'bg-muted-foreground/30'}"
                                >
                                    <span class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform {lobbyHiddenMode ? 'translate-x-5' : ''}" />
                                </button>
                            </div>
                            <Button onclick={lobbyStartGame} class="w-full mt-2" size="lg">Start Game</Button>
                        {:else}
                            <div class="flex items-center justify-between px-3 py-2 rounded-md border border-border bg-card">
                                <span class="text-sm text-foreground">Hidden Mode Only</span>
                                <span class="text-xs text-muted-foreground">{lobbyHiddenMode ? 'On' : 'Off'}</span>
                            </div>
                            <p class="text-sm text-muted-foreground text-center">Waiting for host to start the game...</p>
                        {/if}
                        <Button onclick={lobbyLeaveRoom} variant="outline" class="w-full mt-1">Leave Room</Button>
                    </CardContent>
                </Card>
            {/if}
        </div>

        {#if lobbyRoomId}
        <!-- Chat card (lobby, side panel) -->
        <div class="w-64 shrink-0 flex flex-col">
            <Card class="h-full flex flex-col">
                <CardContent class="flex flex-col gap-2 p-3 flex-1">
                    <div class="text-xs font-medium text-muted-foreground">Chat</div>
                    <div bind:this={chatContainer} class="flex-1 overflow-y-auto space-y-1 scrollbar-thin" style="min-height:120px">
                        {#each chatMessages as msg, i (msg.id)}
                            <div>
                                {#if i === 0 || chatMessages[i-1].playerName !== msg.playerName}
                                    <span class="text-xs font-semibold text-accent">{msg.playerName}</span>
                                    <span class="text-xs text-muted-foreground tabular-nums">{fmtChatTime(msg.timestamp)}</span>
                                {/if}
                                <div class="text-sm text-foreground/90 break-words">{msg.text}</div>
                            </div>
                        {:else}
                            <p class="text-xs text-muted-foreground text-center pt-8">No messages yet</p>
                        {/each}
                    </div>
                    <div class="flex gap-2">
                        <Input
                            bind:value={chatText}
                            onkeydown={chatHandleKey}
                            placeholder="Chat..."
                            maxlength={500}
                            class="flex-1 h-8 text-xs"
                        />
                        <Button onclick={chatSend} size="sm" class="h-8 px-3 text-xs">Send</Button>
                    </div>
                </CardContent>
            </Card>
        </div>
        {/if}
    </div>
{/if}
</div>