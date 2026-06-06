<script lang="ts">
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
        lobbyRoomId = $bindable(""),
        lobbyPlayerId = $bindable(""),
        lobbyMySeatIndex = $bindable(0),
        lobbyIsHost = $bindable(false),
        lobbyPlayers = $bindable<Array<{ name: string; seatIndex: number; isBot: boolean }>>([]),
        lobbyHiddenMode = $bindable(true),
        difficulty = $bindable("Easy"),
        oncreate = () => {},
        onjoin = (_roomId: string) => {},
        onleave = () => {},
        onstart = () => {},
        ontogglehidden = () => {},
    } = $props()

    // Internal lobby UI state
    let lobbyMode = $state<"" | "create" | "join">("create")
    let lobbyCreating = $state(false)
    let lobbyJoinRoomId = $state("")
    let lobbyJoining = $state(false)
    let lobbyJoinError = $state("")

    // Reset loading flags when room is created/joined successfully
    $effect(() => {
        if (lobbyRoomId) {
            lobbyCreating = false
            lobbyJoining = false
        }
    })

    function handleCreate() {
        lobbyCreating = true
        try {
            oncreate()
        } catch (e) {
            lobbyCreating = false
            console.error("Create room error:", e)
        }
    }

    function handleJoin() {
        if (!lobbyJoinRoomId.trim()) return
        lobbyJoining = true
        lobbyJoinError = ""
        try {
            onjoin(lobbyJoinRoomId.trim())
        } catch (e) {
            lobbyJoining = false
            lobbyJoinError = "Failed. Is the backend running?"
            console.error("Join error:", e)
        }
    }

    function handleLeave() {
        try {
            onleave()
        } catch (e) {
            console.error("Leave error:", e)
        }
    }

    function handleStart() {
        try {
            onstart()
        } catch (e) {
            console.error("Start error:", e)
        }
    }

    function handleToggleHidden() {
        try {
            ontogglehidden()
        } catch (e) {
            console.error("Toggle hidden mode error:", e)
        }
    }

    async function copyRoomId() {
        try { await navigator.clipboard.writeText(lobbyRoomId) } catch {}
    }
</script>

<div class="w-full max-w-md">
    {#if !lobbyRoomId}
        <!-- Mode Selector (always visible) -->
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
                    <Button class = "cursor-pointer" onclick={handleCreate} disabled={lobbyCreating}>
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
                    <Button onclick={handleJoin} disabled={lobbyJoining || !lobbyJoinRoomId.trim()}>
                        {lobbyJoining ? "Joining..." : "Join Room"}
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
                <CardDescription class="text-center max-w-[220px] mx-auto">Copy Room ID and invite friends</CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                <div class="flex items-center gap-2 p-3 rounded-lg border border-border bg-muted/50">
                    <span class="text-sm font-mono text-muted-foreground flex-1 truncate">{lobbyRoomId}</span>
                    <Button onclick={copyRoomId} variant="outline" size="sm">Copy</Button>
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
