<script lang="ts">
    import { headerState } from "$lib/game/header-state.svelte";

    let entries: Array<{
        rank: number;
        username: string;
        gamesPlayed: number;
        gamesWon: number;
        winrate: number;
        elo: number;
    }> = $state([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    const API_URL = window.location.hostname === "localhost" || window.location.hostname === "127.0.0.1"
        ? "http://127.0.0.1:3000"
        : "https://bridge-club.duckdns.org";

    async function loadLeaderboard() {
        try {
            const res = await fetch(`${API_URL}/api/leaderboard`);
            const data = await res.json();
            if (data.ok && data.entries) {
                // Backend returns entries sorted by Elo already
                entries = data.entries.map((e: any, i: number) => ({
                    rank: i + 1,
                    username: e.username,
                    gamesPlayed: e.gamesPlayed,
                    gamesWon: e.gamesWon,
                    winrate: e.winrate,
                    elo: e.elo ?? 500,
                }));
            } else {
                error = "Failed to load leaderboard";
            }
        } catch (e) {
            error = "Could not connect to server";
        } finally {
            loading = false;
        }
    }

    $effect(() => { loadLeaderboard(); });
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-1">Leaderboard</h1>
    <p class="text-sm text-muted-foreground mb-8">Ranked by Elo rating</p>

    {#if loading}
        <p class="text-sm text-muted-foreground">Loading...</p>
    {:else if error}
        <p class="text-sm text-red">{error}</p>
    {:else}
        <div class="w-full max-w-2xl mb-10">
            <!-- Header row -->
            <div class="flex items-center px-4 py-2 text-xs text-muted-foreground uppercase tracking-wider border-b border-border">
                <span class="w-10 text-center">#</span>
                <span class="flex-1">Player</span>
                <span class="w-16 text-right">Games</span>
                <span class="w-16 text-right">Won</span>
                <span class="w-16 text-right">Win%</span>
                <span class="w-20 text-right font-bold text-accent">Elo</span>
            </div>

            <!-- Data rows -->
            {#each entries as entry (entry.rank)}
                <div class="flex items-center px-4 py-3 border-b border-border/50 hover:bg-accent/5 transition-colors text-sm">
                    <span class="w-10 text-center font-bold {(headerState.loggedIn && entry.username.toLowerCase() === headerState.username.toLowerCase()) ? 'text-yellow-400' : entry.rank <= 3 ? 'text-accent' : 'text-muted-foreground'}">
                        {entry.rank === 1 ? '🥇' : entry.rank === 2 ? '🥈' : entry.rank === 3 ? '🥉' : entry.rank}
                    </span>
                    <span class="flex-1 {(headerState.loggedIn && entry.username.toLowerCase() === headerState.username.toLowerCase()) ? 'text-yellow-400' : 'text-foreground'}">{entry.username}</span>
                    <span class="w-16 text-right {(headerState.loggedIn && entry.username.toLowerCase() === headerState.username.toLowerCase()) ? 'text-yellow-400' : 'text-foreground'}">{entry.gamesPlayed}</span>
                    <span class="w-16 text-right {(headerState.loggedIn && entry.username.toLowerCase() === headerState.username.toLowerCase()) ? 'text-yellow-400' : 'text-foreground'}">{entry.gamesWon}</span>
                    <span class="w-16 text-right {(headerState.loggedIn && entry.username.toLowerCase() === headerState.username.toLowerCase()) ? 'text-yellow-400' : entry.winrate >= 0.6 ? 'text-green' : 'text-foreground'}">
                        {(entry.winrate * 100).toFixed(0)}%
                    </span>
                    <span class="w-20 text-right font-bold {(headerState.loggedIn && entry.username.toLowerCase() === headerState.username.toLowerCase()) ? 'text-yellow-400' : 'text-accent'}">{entry.elo}</span>
                </div>
            {/each}
        </div>
    {/if}
</div>
