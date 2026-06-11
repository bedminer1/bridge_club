<script lang="ts">
    import { wsClient } from "$lib/game/ws-client";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import * as Select from "$lib/components/ui/select/index.js";
    import { Label } from "$lib/components/ui/label/index.js";

    let {
        onlineToken = "",
        username = "",
        userID = 0,
        lobbyRoomId = $bindable(""),
        lobbyPlayerId = $bindable(""),
        lobbyMySeatIndex = $bindable(0),
        lobbyIsHost = $bindable(false),
        lobbyPlayers = $bindable<Array<{ name: string; seatIndex: number; isBot: boolean }>>([]),
        lobbyHiddenMode = $bindable(true),
        difficulty = $bindable("Easy"),
        onguestlogin = (_username: string, _token: string, _userId: number) => {},
    } = $props()

    // Internal lobby UI state
    let lobbyMode = $state<"" | "create" | "join">("create")
    let lobbyCreating = $state(false)
    let lobbyJoinRoomId = $state("")
    let lobbyJoining = $state(false)
    let lobbyJoinError = $state("")

    // Guest login state
    let guestUsername = $state("")
    let guestChecking = $state(false)
    let guestError = $state("")
    let isGuest = $state(!onlineToken && !username)

    // API URL
    let API_URL: string
    if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
        API_URL = "http://127.0.0.1:3000"
    } else {
        API_URL = "https://bridge-club.duckdns.org"
    }

    // Reset loading flags when room is created/joined successfully
    $effect(() => {
        if (lobbyRoomId) {
            lobbyCreating = false
            lobbyJoining = false
        }
    })

    // Update isGuest when token changes
    $effect(() => {
        isGuest = !onlineToken && !username
    })

    /** Guest login: signup if name available, login if returning guest. */
    async function handleGuestLogin(): Promise<boolean> {
        const name = guestUsername.trim()
        if (!name || name.length < 2) {
            guestError = "Username must be at least 2 characters"
            return false
        }
        guestChecking = true
        guestError = ""
        try {
            // Check if username is available
            const checkRes = await fetch(`${API_URL}/api/auth/check-username?username=${encodeURIComponent(name)}`)
            const checkData = await checkRes.json()
            if (!checkData.ok) {
                guestError = checkData.error || "Failed to check username"
                guestChecking = false
                return false
            }

            let token: string
            let userId: number

            if (checkData.available) {
                // Sign up as a new guest (no password)
                const signupRes = await fetch(`${API_URL}/api/auth/signup`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ username: name, password: "" }),
                })
                const signupData = await signupRes.json()
                if (!signupData.ok) {
                    guestError = signupData.error || "Failed to create account"
                    guestChecking = false
                    return false
                }
                token = signupData.token!
                userId = signupData.user_id!
            } else {
                // Username taken — try logging in as a returning guest
                const loginRes = await fetch(`${API_URL}/api/auth/login`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ username: name, password: "" }),
                })
                const loginData = await loginRes.json()
                if (!loginData.ok) {
                    guestError = loginData.error || "Username taken"
                    guestChecking = false
                    return false
                }
                token = loginData.token!
                userId = loginData.user_id!
            }

            document.cookie = `session=${token}; path=/; max-age=2592000; SameSite=Lax`
            onguestlogin(name, token, userId)
            guestChecking = false
            return true
        } catch {
            guestError = "Connection error. Is the backend running?"
            guestChecking = false
            return false
        }
    }

    // ── Lobby actions (call wsClient directly) ─────────────────────

    function doCreateRoom() {
        try {
            wsClient.createLobby()
        } catch (e) { console.error("Create room error:", e); alert("Failed. Is the backend running?") }
    }

    function doJoinRoom(joinRoomId: string) {
        if (!joinRoomId.trim()) return
        try {
            wsClient.joinLobby(joinRoomId.trim())
        } catch (e) { console.error("Join error:", e); alert("Failed. Is the backend running?") }
    }

    function doLeaveRoom() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        try {
            wsClient.leaveLobby()
        } catch (e) { console.error("Leave error:", e) }
        lobbyRoomId = ""; lobbyPlayerId = ""; lobbyMySeatIndex = 0; lobbyIsHost = false; lobbyPlayers = []
        try { localStorage.removeItem("bridgeActiveRoom") } catch {}
    }

    function doStartGame() {
        try {
            wsClient.startGame(lobbyHiddenMode, difficulty || "Easy")
        } catch (e) { console.error("Start error:", e); alert("Failed. Is the backend running?") }
    }

    function doToggleHidden() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        try {
            wsClient.toggleHidden(!lobbyHiddenMode)
        } catch (e) { console.error("Toggle hidden mode error:", e) }
    }

    // ── UI handlers ───────────────────────────────────────────────

    function handleCreate() {
        if (isGuest) {
            handleGuestLogin().then(ok => { if (ok) { lobbyCreating = true; doCreateRoom() } }).catch(() => {})
            return
        }
        lobbyCreating = true
        doCreateRoom()
    }

    function handleJoin() {
        if (!lobbyJoinRoomId.trim()) return
        if (isGuest) {
            handleGuestLogin().then(ok => { if (ok) { lobbyJoining = true; lobbyJoinError = ""; doJoinRoom(lobbyJoinRoomId.trim()) } }).catch(() => {})
            return
        }
        lobbyJoining = true
        lobbyJoinError = ""
        doJoinRoom(lobbyJoinRoomId.trim())
    }

    function handleLeave() {
        doLeaveRoom()
    }

    function handleStart() {
        doStartGame()
    }

    function handleToggleHidden() {
        doToggleHidden()
    }

    let shareUrl = $derived(lobbyRoomId
        ? `${typeof window !== 'undefined' ? window.location.origin : ''}/?room=${lobbyRoomId}`
        : "")

    let copied = $state(false)

    async function copyShareUrl() {
        try {
            await navigator.clipboard.writeText(shareUrl)
            copied = true
            setTimeout(() => copied = false, 1500)
        } catch {}
    }
</script>

<div class="w-full max-w-md">
    {#if !lobbyRoomId}
        <!-- Mode Selector -->
        <div class="relative flex justify-center mb-6 items-center">
            {#if lobbyMode}
            <button onclick={() => lobbyMode = ""} class="absolute left-0 p-1.5 rounded-md text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors text-4xl leading-none" title="Back">
                &larr;
            </button>
            {/if}
            <div class="flex gap-2">
                <Button class = "cursor-pointer" onclick={() => { if (lobbyMode === "create") handleCreate(); else lobbyMode = "create" }}
                    variant={lobbyMode === "create" ? "default" : "outline"}>
                    Create Room
                </Button>
                <Button class="cursor-pointer" onclick={() => { lobbyMode = "join" }}
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
                    {#if isGuest}
                        <div class="flex flex-col gap-2">
                            <label for="guest-username" class="text-xs text-muted-foreground">Username</label>
                            <Input
                                id="guest-username"
                                bind:value={guestUsername}
                                placeholder="Enter a username"
                                onkeydown={(e) => { if (e.key === 'Enter') handleCreate() }}
                                disabled={guestChecking}
                            />
                        </div>
                        {#if guestError}
                            <p class="text-xs text-destructive">{guestError}</p>
                        {/if}
                        <p class="text-[10px] text-muted-foreground text-center">
                            · <a href="/login" class="text-accent hover:underline">log in</a> if returning
                        </p>
                    {/if}
                    <Button class = "cursor-pointer" onclick={handleCreate} disabled={lobbyCreating || (isGuest && guestChecking)}>
                        {lobbyCreating ? "Creating..." : isGuest ? "Create Account & Room" : "Create Room"}
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
                        {#if isGuest}
                            <div class="flex flex-col gap-2">
                                <label for="guest-username-join" class="text-xs text-muted-foreground">Username</label>
                                <Input
                                    id="guest-username-join"
                                    bind:value={guestUsername}
                                    placeholder="Enter a username"
                                    onkeydown={(e) => { if (e.key === 'Enter') handleJoin() }}
                                />
                            </div>
                            {#if guestError}
                                <p class="text-xs text-destructive">{guestError}</p>
                            {/if}
                            <p class="text-[10px] text-muted-foreground text-center">
                                · <a href="/login" class="text-accent hover:underline">log in</a> if returning
                            </p>
                        {/if}
                        <div class="flex flex-col gap-2">
                            <label for="room-id" class="text-sm text-muted-foreground">Room ID</label>
                            <Input id="room-id" bind:value={lobbyJoinRoomId} placeholder="Enter room ID" />
                        </div>
                        {#if lobbyJoinError}<p class="text-sm text-destructive">{lobbyJoinError}</p>{/if}
                        <Button onclick={handleJoin} disabled={lobbyJoining || !lobbyJoinRoomId.trim() || (isGuest && guestChecking)}>
                            {lobbyJoining ? "Joining..." : isGuest ? "Create Account & Join" : "Join Room"}
                        </Button>
                    </CardContent>
                </Card>
            {/if}
    {:else}
        <!-- Lobby View (waiting room) -->
        <Card>
            <CardHeader class="relative">
                <button onclick={handleLeave} class="absolute left-3 inset-y-0 flex items-center text-3xl leading-none text-muted-foreground hover:text-foreground transition-colors" title="Leave room">
                    &larr;
                </button>
                <CardTitle class="text-center">Game Lobby</CardTitle>
                <CardDescription class="text-center max-w-[220px] mx-auto">Share the invite link with friends</CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                <div class="flex items-center gap-2 p-3 rounded-lg border border-border bg-muted/50">
                    <span class="flex-1 text-xs text-muted-foreground truncate">{shareUrl}</span>
                    <button onclick={copyShareUrl} class="shrink-0 p-1 rounded text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors" title="Copy invite link">
                        {#if copied}
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-green"><polyline points="20 6 9 17 4 12"/></svg>
                        {:else}
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                        {/if}
                    </button>
                </div>
                <div class="flex flex-col gap-2">
                    <h3 class="text-sm font-medium text-foreground">Players ({lobbyPlayers.length}/4)</h3>
                    <div class="flex flex-col gap-1">
                        {#each [0, 1, 2, 3] as seatIdx}
                            {@const player = lobbyPlayers.find((p: any) => p.seatIndex === seatIdx)}
                            <div class="flex items-center gap-2 px-3 py-2 rounded-md border {player ? 'border-border bg-card' : 'border-dashed border-border/40 bg-muted/20'}">
                                <div class="w-2 h-2 rounded-full {player ? (player.isBot ? 'bg-muted-foreground/40' : 'bg-green-500') : 'bg-border/30'}"></div>
                                {#if player}
                                    <span class="text-sm text-foreground">{player.name}</span>
                                    {#if player.isBot}<span class="text-xs text-muted-foreground">(bot)</span>{/if}
                                {:else}
                                    <span class="text-sm text-muted-foreground/40 italic">Waiting...</span>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
                {#if lobbyIsHost}
                    <div class="flex items-center justify-between px-3 py-2 rounded-md border border-border bg-card">
                        <Label for="difficulty">Difficulty</Label>
                        <Select.Root type="single" bind:value={difficulty}>
                            <Select.Trigger class="w-[100px]">{difficulty}</Select.Trigger>
                            <Select.Content>
                                <Select.Item value="Easy">Easy</Select.Item>
                                <Select.Item value="Medium">Medium</Select.Item>
                                <Select.Item value="Hard" disabled>Hard</Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>
                    <div class="flex items-center justify-between px-3 py-2 rounded-md border border-border bg-card">
                        <span class="text-sm text-foreground">Tutorial Mode</span>
                        <button
                            onclick={handleToggleHidden}
                            aria-label="Toggle tutorial mode"
                            class="relative w-10 h-5 rounded-full transition-colors {!lobbyHiddenMode ? 'bg-accent' : 'bg-muted-foreground/30'}"
                        >
                            <span class="cursor-pointer absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform {!lobbyHiddenMode ? 'translate-x-5' : ''}"></span>
                        </button>
                    </div>
                    <Button onclick={handleStart} class="cursor-pointer w-full mt-2" size="lg">Start Game</Button>
                {:else}
                    <div class="flex items-center justify-between px-3 py-2 rounded-md border border-border bg-card">
                        <span class="text-sm text-foreground">Tutorial Mode</span>
                        <span class="text-xs text-muted-foreground">{lobbyHiddenMode ? 'Off' : 'On'}</span>
                    </div>
                    <p class="text-sm text-muted-foreground text-center">Waiting for host to start the game...</p>
                {/if}
                <Button onclick={handleLeave} variant="outline" class="cursor-pointer w-full mt-1">Leave Room</Button>
            </CardContent>
        </Card>
    {/if}
</div>
