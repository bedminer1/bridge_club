<script lang="ts">
    /** Dummy leaderboard data — replace with real API later */
    const entries = [
        { rank: 1, username: "cardcounter", games: 142, winRate: 68, avgSets: 7.3, best: 9 },
        { rank: 2, username: "trumpKing", games: 98, winRate: 65, avgSets: 7.1, best: 9 },
        { rank: 3, username: "spadeAce", games: 211, winRate: 62, avgSets: 6.9, best: 8 },
        { rank: 4, username: "bridgeBot", games: 56, winRate: 59, avgSets: 6.7, best: 8 },
        { rank: 5, username: "dummyHand", games: 173, winRate: 57, avgSets: 6.6, best: 8 },
        { rank: 6, username: "grandSlam", games: 34, winRate: 55, avgSets: 6.5, best: 7 },
        { rank: 7, username: "finesse", games: 87, winRate: 53, avgSets: 6.4, best: 7 },
        { rank: 8, username: "noTrump", games: 42, winRate: 50, avgSets: 6.2, best: 7 },
        { rank: 9, username: "oakLeaf", games: 25, winRate: 48, avgSets: 6.0, best: 7 },
        { rank: 10, username: "caliburACE", games: 19, winRate: 42, avgSets: 5.8, best: 6 },
    ]
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-1">Leaderboard</h1>
    <p class="text-sm text-muted-foreground mb-8">Top players by win rate</p>

    <div class="w-full max-w-2xl">
        <!-- Header row -->
        <div class="flex items-center px-4 py-2 text-xs text-muted-foreground uppercase tracking-wider border-b border-border">
            <span class="w-10 text-center">#</span>
            <span class="flex-1">Player</span>
            <span class="w-16 text-right">Games</span>
            <span class="w-16 text-right">Win%</span>
            <span class="w-16 text-right">Avg</span>
            <span class="w-16 text-right">Best</span>
        </div>

        <!-- Data rows -->
        {#each entries as entry (entry.rank)}
            <div class="flex items-center px-4 py-3 border-b border-border/50 hover:bg-accent/5 transition-colors text-sm">
                <span class="w-10 text-center font-bold {entry.rank <= 3 ? 'text-accent' : 'text-muted-foreground'}">
                    {entry.rank === 1 ? '🥇' : entry.rank === 2 ? '🥈' : entry.rank === 3 ? '🥉' : entry.rank}
                </span>
                <span class="flex-1 text-foreground">{entry.username}</span>
                <span class="w-16 text-right text-muted-foreground">{entry.games}</span>
                <span class="w-16 text-right {entry.winRate >= 60 ? 'text-green' : 'text-muted-foreground'}">
                    {entry.winRate}%
                </span>
                <span class="w-16 text-right text-muted-foreground">{entry.avgSets.toFixed(1)}</span>
                <span class="w-16 text-right text-accent">{entry.best}</span>
            </div>
        {/each}
    </div>
</div>
