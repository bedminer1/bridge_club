<script lang="ts">
    import { Gamepad2, Crown, Github } from "@lucide/svelte"
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-2">Bridge Club</h1>
    <p class="text-sm text-muted-foreground mb-8">Singapore Bridge — Hidden Mode Only — Elo ranked</p>

    <div class="w-full max-w-md flex flex-col gap-4 text-sm">
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-2">How to Play</h2>
            <ul class="space-y-2 text-muted-foreground">
                <li><span class="text-accent">Betting</span> — each player bids once, starting left of dealer. Bid how many tricks (sets) your team can take, paired with a trump suit. Higher suit rank (♣&lt;♦&lt;♥&lt;♠) breaks equal-level bids. Pass if you don't want to raise. The highest bidder wins the contract.</li>
                <li><span class="text-accent">Partner</span> — the bet winner picks a card from the remaining deck. The player who holds that card becomes their partner for the round. The two partners form Team 1; the other two form Team 2.</li>
                <li><span class="text-accent">Play</span> — the player left of the dealer leads the first trick. Follow suit if you can; otherwise play anything. Trump cannot be led until it has been played on a trick (broken). If void in the led suit, you may play trump even before it's been broken.</li>
                <li><span class="text-accent">Scoring</span> — a trick is won by the highest card of the led suit, or by the highest trump if any. Team 1 wins if they take 6 + bet_size tricks; Team 2 wins if they take 14 &minus; bet_size tricks.</li>
                <li><span class="text-accent">Elo</span> — rated matches use team-average Elo with K=32. Hidden Mode Only games count; Open games are for practice (no rating change).</li>
            </ul>
        </div>

        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-2">About</h2>
            <p class="text-muted-foreground">
                Built as a learning project. Open source on
                <a href="https://github.com/bedminer1/bridge_club" target="_blank" class="text-accent hover:underline">
                    GitHub <Github class="inline w-3 h-3" />
                </a>.
            </p>
        </div>

        <div class="flex gap-4 justify-center mt-4">
            <a href="/" class="flex items-center gap-2 text-sm text-accent hover:underline">
                <Gamepad2 class="w-4 h-4" /> Play
            </a>
            <a href="/leaderboard" class="flex items-center gap-2 text-sm text-accent hover:underline">
                <Crown class="w-4 h-4" /> Leaderboard
            </a>
        </div>
    </div>
</div>
