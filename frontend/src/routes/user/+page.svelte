<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js"
    import ScoreDisplay from "./ScoreDisplay.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import { formatDate } from "$lib/utils";
    import { headerState } from "$lib/game/header-state.svelte";
    import { toggleMode } from "mode-watcher";
    import { Switch } from "$lib/components/ui/switch/index.js";

    let { data } = $props()
    let { matchRecords, message, username, userStats } = $state(data)

    // Re-fetch user stats on mount to ensure latest data
    $effect(() => {
        if (message === "success") {
            // Try to get fresh session data from the backend
            const token = document.cookie.split("; ").find(r => r.startsWith("session="))
            if (token) {
                fetch("https://bridge-club.duckdns.org/api/auth/session?token=" + encodeURIComponent(token.split("=")[1]))
                    .then(r => r.json())
                    .then(d => {
                        if (d.ok && d.user) {
                            userStats = {
                                gamesPlayed: d.user.gamesPlayed,
                                gamesWon: d.user.gamesWon,
                                totalSetsWon: d.user.totalSetsWon,
                                mostSetsWon: d.user.mostSetsWon,
                            }
                        }
                    })
                    .catch(() => {})
            }
        }
    })

    /** Parse the players field from a match record, returning per-player names. */
    function getPlayerName(matchRecord: any, seatIndex: number): string {
        try {
            const players = JSON.parse(matchRecord.players || "[]")
            return players[seatIndex]?.username ?? `P${seatIndex + 1}`
        } catch {
            return `P${seatIndex + 1}`
        }
    }

    $effect(() => { headerState.username = username ?? "" })
    $effect(() => { headerState.loggedIn = message === "success" })

    let winrate = $derived(userStats?.gamesPlayed > 0 ? ((userStats.gamesWon / userStats.gamesPlayed) * 100).toFixed(1) : "0.0")
    let avgSets = $derived(userStats?.gamesPlayed > 0 ? (userStats.totalSetsWon / userStats.gamesPlayed).toFixed(1) : "0.0")
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    {#if message !== "success"}
        <p class="text-lg text-muted-foreground mt-20">
            <a href="/login" class="text-accent hover:underline">{message}</a>
        </p>
    {:else}
        <!-- User header -->
        <div class="flex items-center gap-3 mb-6">
            <div class="w-10 h-10 rounded-full bg-accent/20 flex items-center justify-center text-accent font-bold text-lg">
                {username?.charAt(0).toUpperCase() ?? "?"}
            </div>
            <div>
                <h1 class="text-xl font-bold">{username}</h1>
                <p class="text-xs text-muted-foreground">{matchRecords.length} match{matchRecords.length !== 1 ? "es" : ""} played</p>
            </div>
        </div>

        <!-- Stats cards -->
        {#if userStats}
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 w-full max-w-2xl mb-6">
                <Card.Root class="rounded-lg border-border p-4 text-center">
                    <p class="text-xs text-muted-foreground uppercase tracking-wider mb-1">Games Played</p>
                    <p class="text-2xl font-bold text-foreground">{userStats.gamesPlayed}</p>
                </Card.Root>
                <Card.Root class="rounded-lg border-border p-4 text-center">
                    <p class="text-xs text-muted-foreground uppercase tracking-wider mb-1">Games Won</p>
                    <p class="text-2xl font-bold text-green">{userStats.gamesWon}</p>
                </Card.Root>
                <Card.Root class="rounded-lg border-border p-4 text-center">
                    <p class="text-xs text-muted-foreground uppercase tracking-wider mb-1">Win Rate</p>
                    <p class="text-2xl font-bold text-foreground">{winrate}%</p>
                </Card.Root>
                <Card.Root class="rounded-lg border-border p-4 text-center">
                    <p class="text-xs text-muted-foreground uppercase tracking-wider mb-1">Avg Sets</p>
                    <p class="text-2xl font-bold text-foreground">{avgSets}</p>
                </Card.Root>
                <Card.Root class="rounded-lg border-border p-4 text-center">
                    <p class="text-xs text-muted-foreground uppercase tracking-wider mb-1">Total Sets Won</p>
                    <p class="text-2xl font-bold text-foreground">{userStats.totalSetsWon}</p>
                </Card.Root>
                <Card.Root class="rounded-lg border-border p-4 text-center">
                    <p class="text-xs text-muted-foreground uppercase tracking-wider mb-1">Most Sets (Best)</p>
                    <p class="text-2xl font-bold text-accent">{userStats.mostSetsWon}</p>
                </Card.Root>
            </div>
        {/if}

        <!-- Logout -->
        <form action="?/logout" method="POST" class="mb-4">
            <button class="text-xs text-muted-foreground hover:text-destructive transition-colors underline underline-offset-2">
                logout
            </button>
        </form>

        <!-- Theme toggle -->
        <div class="flex items-center gap-3 mb-10">
            <span class="text-xs text-muted-foreground">Light Mode</span>
            <Switch bind:checked={headerState.isLightMode} onclick={toggleMode} />
        </div>

        <!-- Match history -->
        <div class="w-full max-w-3xl">
            <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-4">Match History</h2>

            {#if matchRecords.length > 0}
                <div class="flex flex-col gap-3">
                    {#each matchRecords as matchRecord}
                        <a href="/user/{matchRecord.id}" class="block">
                            <Card.Root class="w-full rounded-lg border-border hover:border-accent/30 transition-colors">
                                <div class="flex items-center justify-between p-4 gap-4">
                                    <!-- Result badge -->
                                    <div class="w-16 shrink-0">
                                        <span
                                            class="text-sm font-bold {matchRecord.wonMatch ? 'text-green' : 'text-red'}"
                                        >
                                            {matchRecord.wonMatch ? "Victory" : "Defeat"}
                                        </span>
                                    </div>

                                    <!-- Info -->
                                    <div class="flex-1 min-w-0">
                                        <div class="flex gap-3 mb-5 text-xs text-muted-foreground">
                                            <span>{matchRecord.betSize}{matchRecord.trumpSuit.toUpperCase()}</span>
                                            <span>|</span>
                                            <span>{matchRecord.botDifficulty}</span>
                                            <span>|</span>
                                            <span>{formatDate(matchRecord.date)}</span>
                                        </div>
                                        <div class="flex ml-4">
                                            {#each JSON.parse(matchRecord.player1Hand) as card, index}
                                                <HandDisplay {index}>
                                                    <PokerCard card={card} isIllegal={false} minify={true} />
                                                </HandDisplay>
                                            {/each}
                                        </div>
                                    </div>

                                    <!-- Scores -->
                                    <div class="hidden sm:block shrink-0">
                                        <ScoreDisplay {matchRecord} />
                                    </div>
                                </div>
                            </Card.Root>
                        </a>
                    {/each}
                </div>
            {:else}
                <p class="text-sm text-muted-foreground">No matches played yet.</p>
            {/if}
        </div>
    {/if}
</div>
