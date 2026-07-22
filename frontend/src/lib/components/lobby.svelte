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
    import * as Select from "$lib/components/ui/select/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { goto } from "$app/navigation";

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
    } = $props()

    let lobbyCreating = $state(false)
    let lobbyStartingSolo = $state(false)
    let loggedIn = $derived(!!onlineToken && userID > 0)

    // Reset loading when room is created
    $effect(() => {
        if (lobbyRoomId) {
            lobbyCreating = false
            lobbyStartingSolo = false
        }
    })

    function doCreateRoom() {
        try {
            wsClient.createLobby()
        } catch (e) { console.error("Create room error:", e); alert("Failed. Is the backend running?") }
    }

    function doLeaveRoom() {
        if (!lobbyRoomId || !lobbyPlayerId) return
        try { wsClient.leaveLobby() } catch (e) { console.error("Leave error:", e) }
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
        try { wsClient.toggleHidden(!lobbyHiddenMode) } catch (e) { console.error("Toggle hidden mode error:", e) }
    }

    function handleCreate() {
        if (!loggedIn) {
            goto("/login")
            return
        }
        lobbyCreating = true
        doCreateRoom()
    }

    function handleSinglePlayer() {
        if (!loggedIn) {
            goto("/login")
            return
        }
        lobbyStartingSolo = true
        try {
            wsClient.createLobby()
            wsClient.startGame(lobbyHiddenMode, difficulty || "Easy")
        } catch (e) {
            console.error("Single-player start error:", e)
            alert("Failed. Is the backend running?")
            lobbyStartingSolo = false
        }
    }

    function handleLeave() { doLeaveRoom() }
    function handleStart() { doStartGame() }
    function handleToggleHidden() { doToggleHidden() }

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
        <Card>
            <CardHeader>
                <CardTitle class="text-center text-2xl">Welcome to Bridge Club!</CardTitle>
                <CardDescription class="text-center max-w-[220px] mx-auto">
                    {#if loggedIn}
                        Create a room and invite friends
                    {:else}
                        Sign in to start playing
                    {/if}
                </CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                {#if loggedIn}
                    <Button class="cursor-pointer" onclick={handleCreate} disabled={lobbyCreating || lobbyStartingSolo} size="lg">
                        {lobbyCreating ? "Creating lobby..." : "Multiplayer"}
                    </Button>
                    <Button class="cursor-pointer" onclick={handleSinglePlayer} disabled={lobbyCreating || lobbyStartingSolo} size="lg" variant="outline">
                        {lobbyStartingSolo ? "Starting..." : "Single Player"}
                    </Button>
                {:else}
                    <Button class="cursor-pointer" onclick={() => goto("/login")} size="lg">
                        Sign in to Play
                    </Button>
                {/if}
            </CardContent>
        </Card>
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
