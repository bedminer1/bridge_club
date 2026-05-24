<script lang="ts">
    let entries: Array<{
        rank: number;
        username: string;
        gamesPlayed: number;
        gamesWon: number;
        winrate: number;
        totalSetsWon: number;
        mostSetsWon: number;
    }> = $state([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    async function loadLeaderboard() {
        try {
            const res = await fetch("https://bridge-club.duckdns.org/api/leaderboard");
            const data = await res.json();
            if (data.ok && data.entries) {
                // Sort by winrate descending for display
                const sorted = [...data.entries].sort((a, b) => b.winrate - a.winrate || b.gamesPlayed - a.gamesPlayed);
                entries = sorted.map((e: any, i: number) => ({
                    rank: i + 1,
                    username: e.username,
                    gamesPlayed: e.gamesPlayed,
                    gamesWon: e.gamesWon,
                    winrate: e.winrate,
                    totalSetsWon: e.totalSetsWon,
                    mostSetsWon: e.mostSetsWon,
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
    <p class="text-sm text-muted-foreground mb-8">Top players by win rate</p>

    {#if loading}
        <p class="text-sm text-muted-foreground">Loading...</p>
    {:else if error}
        <p class="text-sm text-red">{error}</p>
    {:else}
        <div class="w-full max-w-2xl">
            <!-- Header row -->
            <div class="flex items-center px-4 py-2 text-xs text-muted-foreground uppercase tracking-wider border-b border-border">
                <span class="w-10 text-center">#</span>
                <span class="flex-1">Player</span>
                <span class="w-16 text-right">Games</span>
                <span class="w-16 text-right">Won</span>
                <span class="w-16 text-right">Win%</span>
                <span class="w-16 text-right">Avg Sets</span>
                <span class="w-16 text-right">Best</span>
            </div>

            <!-- Data rows -->
            {#each entries as entry (entry.rank)}
                <div class="flex items-center px-4 py-3 border-b border-border/50 hover:bg-accent/5 transition-colors text-sm">
                    <span class="w-10 text-center font-bold {entry.rank <= 3 ? 'text-accent' : 'text-muted-foreground'}">
                        {entry.rank === 1 ? '🥇' : entry.rank === 2 ? '🥈' : entry.rank === 3 ? '🥉' : entry.rank}
                    </span>
                    <span class="flex-1 text-foreground">{entry.username}</span>
                    <span class="w-16 text-right text-muted-foreground">{entry.gamesPlayed}</span>
                    <span class="w-16 text-right text-muted-foreground">{entry.gamesWon}</span>
                    <span class="w-16 text-right {entry.winrate >= 0.6 ? 'text-green' : 'text-muted-foreground'}">
                        {(entry.winrate * 100).toFixed(0)}%
                    </span>
                    <span class="w-16 text-right text-muted-foreground">
                        {entry.gamesPlayed > 0 ? (entry.totalSetsWon / entry.gamesPlayed).toFixed(1) : '0.0'}
                    </span>
                    <span class="w-16 text-right text-accent">{entry.mostSetsWon}</span>
                </div>
            {/each}
        </div>
    {/if}
</div>
