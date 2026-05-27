<script lang="ts">
    import ScoreDisplay from "./ScoreDisplay.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import * as Card from "$lib/components/ui/card/index"
    import { formatDate } from "$lib/utils";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { headerState } from "$lib/game/header-state.svelte";
    import { toggleMode } from "mode-watcher";
    import { Switch } from "$lib/components/ui/switch/index.js";

    let { data } = $props()
    let { matchRecords, message, username, rank, userStats } = $state(data)

    // Default stats to 0 when null (not logged in)
    let s = $derived(userStats ?? { gamesPlayed: 0, gamesWon: 0, totalSetsWon: 0, mostSetsWon: 0 })

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
                                gamesPlayed: d.user.gamesPlayed ?? 0,
                                gamesWon: d.user.gamesWon ?? 0,
                                totalSetsWon: d.user.totalSetsWon ?? 0,
                                mostSetsWon: d.user.mostSetsWon ?? 0,
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

    let winrate = $derived(s.gamesPlayed > 0 ? ((s.gamesWon / s.gamesPlayed) * 100).toFixed(1) : "0.0")
    let avgSets = $derived(s.gamesPlayed > 0 ? (s.totalSetsWon / s.gamesPlayed).toFixed(1) : "0.0")
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
                <p class="text-xs text-muted-foreground">{rank > 0 ? `#${rank} Global` : ""}</p>
            </div>
        </div>

        <!-- Stats strip -->
        {#if userStats}
            <div class="w-full max-w-2xl">
                <Separator class="mb-3" />
                <div class="flex justify-center gap-4 sm:gap-6 text-sm text-muted-foreground flex-wrap">
                    <span><strong class="text-foreground">{s.gamesPlayed}</strong> played</span>
                    <span><strong class="text-green">{s.gamesWon}</strong> won</span>
                    <span><strong class="text-foreground">{winrate}%</strong> win rate</span>
                </div>
                <Separator class="mt-3" />
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
                                            class="text-sm font-bold {matchRecord.winningTeam === 1 && (matchRecord.betWinnerUserId === userID || matchRecord.partnerUserId === userID) ? 'text-green' : (!matchRecord.winningTeam || matchRecord.winningTeam === 2 && (matchRecord.betWinnerUserId === userID || matchRecord.partnerUserId === userID) || matchRecord.winningTeam === 1 && matchRecord.betWinnerUserId !== userID && matchRecord.partnerUserId !== userID) ? 'text-red' : ''}"
                                        >
                                            {#if matchRecord.betWinnerUserId === userID || matchRecord.partnerUserId === userID}
                                                {matchRecord.winningTeam === 1 ? "Victory" : "Defeat"}
                                            {:else}
                                                {matchRecord.winningTeam === 2 ? "Victory" : "Defeat"}
                                            {/if}
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
