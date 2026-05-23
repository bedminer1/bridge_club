<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card/index.js";
    import { goto } from "$app/navigation";

    let { data } = $props();
    let { username, token } = $state(data);

    const API_URL = "http://127.0.0.1:3000";

    // Mode: "create" or "join"
    let mode = $state<"create" | "join">("create");

    // Create room state
    let creating = $state(false);
    let isHost = $state(false);

    // Join room state
    let joinRoomId = $state("");
    let joining = $state(false);
    let joinError = $state("");

    // Lobby info (players list)
    let roomId = $state("");
    let mySeatIndex = $state(0);
    let players = $state<Array<{ name: string; seatIndex: number; isBot: boolean }>>([]);
    let pollInterval: ReturnType<typeof setInterval> | null = null;

    // Actions
    async function createRoom() {
        creating = true;
        try {
            const res = await fetch(`${API_URL}/api/rooms`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "X-Session-Token": token,
                },
            });
            if (!res.ok) {
                const text = await res.text().catch(() => "");
                alert(`Failed to create room: ${res.status} ${text}`);
                return;
            }
            const data = await res.json();
            roomId = data.roomId;
            mySeatIndex = data.seatIndex;
            isHost = true;
            startPolling();
        } catch (e) {
            console.error("Create room error:", e);
            alert("Failed to create room. Is the backend running?");
        } finally {
            creating = false;
        }
    }

    async function joinRoom() {
        if (!joinRoomId.trim()) return;
        joining = true;
        joinError = "";
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(joinRoomId.trim())}/join`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "X-Session-Token": token,
                },
            });
            if (!res.ok) {
                const text = await res.text().catch(() => "");
                joinError = `Failed to join: ${res.status} ${text}`;
                return;
            }
            const data = await res.json();
            roomId = data.roomId;
            mySeatIndex = data.seatIndex;
            isHost = false;
            startPolling();
            return;
        } catch (e) {
            console.error("Join room error:", e);
            joinError = "Failed to join room. Is the backend running?";
        } finally {
            joining = false;
        }
    }

    async function startGame() {
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(roomId)}/start`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "X-Session-Token": token,
                },
            });
            if (!res.ok) {
                const text = await res.text().catch(() => "");
                alert(`Failed to start game: ${res.status} ${text}`);
                return;
            }
            const data = await res.json();
            if (data.ok) {
                stopPolling();
                goto(`/?room=${encodeURIComponent(roomId)}&seat=${mySeatIndex}`);
            } else {
                alert("Failed to start game: " + JSON.stringify(data));
            }
        } catch (e) {
            console.error("Start game error:", e);
            alert("Failed to start game. Is the backend running?");
        }
    }

    function startPolling() {
        stopPolling();
        poll();
        pollInterval = setInterval(poll, 2000);
    }

    function stopPolling() {
        if (pollInterval !== null) {
            clearInterval(pollInterval);
            pollInterval = null;
        }
    }

    async function poll() {
        if (!roomId) return;
        try {
            const res = await fetch(`${API_URL}/api/rooms/${encodeURIComponent(roomId)}/info`, {
                headers: {
                    "X-Session-Token": token,
                },
            });
            if (!res.ok) return;
            const data = await res.json();
            if (data.isStarted) {
                // Game already started, redirect
                stopPolling();
                goto(`/?room=${encodeURIComponent(roomId)}&seat=${mySeatIndex}`);
                return;
            }
            players = data.players || [];
        } catch (e) {
            console.error("Poll lobby error:", e);
        }
    }

    async function copyRoomId() {
        try {
            await navigator.clipboard.writeText(roomId);
        } catch {
            // Fallback: select the text manually
        }
    }

    // Cleanup polling on destroy
    $effect(() => {
        return () => stopPolling();
    });
</script>

<div class="flex flex-col gap-6 w-full min-h-screen items-center px-4 pt-20 pb-8">
    <div class="w-full max-w-md">
        {#if !roomId}
            <!-- Mode Selector -->
            <div class="flex justify-center gap-2 mb-6">
                <Button
                    onclick={() => { mode = "create" }}
                    variant={mode === "create" ? "default" : "outline"}
                >
                    Create Room
                </Button>
                <Button
                    onclick={() => { mode = "join" }}
                    variant={mode === "join" ? "default" : "outline"}
                >
                    Join Room
                </Button>
            </div>

            {#if mode === "create"}
                <Card>
                    <CardHeader>
                        <CardTitle>Create a Room</CardTitle>
                        <CardDescription>
                            Create a new lobby and invite friends to play.
                        </CardDescription>
                    </CardHeader>
                    <CardContent class="flex flex-col gap-4">
                        <Button onclick={createRoom} disabled={creating}>
                            {creating ? "Creating..." : "Create Room"}
                        </Button>
                    </CardContent>
                </Card>
            {:else}
                <Card>
                    <CardHeader>
                        <CardTitle>Join a Room</CardTitle>
                        <CardDescription>
                            Enter the room ID shared by the host.
                        </CardDescription>
                    </CardHeader>
                    <CardContent class="flex flex-col gap-4">
                        <div class="flex flex-col gap-2">
                            <label for="room-id" class="text-sm text-muted-foreground">Room ID</label>
                            <Input id="room-id" bind:value={joinRoomId} placeholder="Enter room ID" />
                        </div>
                        {#if joinError}
                            <p class="text-sm text-destructive">{joinError}</p>
                        {/if}
                        <Button onclick={joinRoom} disabled={joining || !joinRoomId.trim()}>
                            {joining ? "Joining..." : "Join Room"}
                        </Button>
                    </CardContent>
                </Card>
            {/if}
        {:else}
            <!-- Lobby View -->
            <Card>
                <CardHeader>
                    <CardTitle>Game Lobby</CardTitle>
                    <CardDescription>
                        Share the room ID below with friends to invite them.
                    </CardDescription>
                </CardHeader>
                <CardContent class="flex flex-col gap-4">
                    <!-- Room ID display -->
                    <div class="flex items-center gap-2 p-3 rounded-lg border border-border bg-muted/50">
                        <span class="text-sm font-mono text-muted-foreground flex-1 truncate">{roomId}</span>
                        <Button onclick={copyRoomId} variant="outline" size="sm">
                            Copy
                        </Button>
                    </div>

                    <!-- Players list -->
                    <div class="flex flex-col gap-2">
                        <h3 class="text-sm font-medium text-foreground">
                            Players ({players.length})
                        </h3>
                        {#if players.length === 0}
                            <p class="text-sm text-muted-foreground">Waiting for players...</p>
                        {:else}
                            <div class="flex flex-col gap-1">
                                {#each players as p}
                                    <div class="flex items-center gap-2 px-3 py-2 rounded-md border border-border bg-card">
                                        <div class="w-2 h-2 rounded-full {p.isBot ? 'bg-muted-foreground/40' : 'bg-green-500'}"></div>
                                        <span class="text-sm text-foreground">{p.name}</span>
                                        {#if p.isBot}
                                            <span class="text-xs text-muted-foreground">(bot)</span>
                                        {/if}
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>

                    <!-- Start button (host only) -->
                    {#if isHost}
                        <Button onclick={startGame} class="w-full mt-2" size="lg">
                            Start Game
                        </Button>
                    {:else}
                        <p class="text-sm text-muted-foreground text-center">
                            Waiting for host to start the game...
                        </p>
                    {/if}
                </CardContent>
            </Card>
        {/if}
    </div>
</div>
