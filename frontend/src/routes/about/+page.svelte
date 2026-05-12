<script lang="ts">
    import { Gamepad2, Crown, Github } from "@lucide/svelte"
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-2">Bridge Club</h1>
    <p class="text-sm text-muted-foreground mb-8">Singapore Bridge — play against bots, track your stats</p>

    <div class="w-full max-w-md flex flex-col gap-4 text-sm">
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-2">How to Play</h2>
            <ul class="space-y-1.5 text-muted-foreground">
                <li><span class="text-accent">Betting</span> — bid on how many sets your team can take. Higher suit = higher priority.</li>
                <li><span class="text-accent">Partner</span> — the bet winner selects a partner card. Whoever holds it is your teammate.</li>
                <li><span class="text-accent">Play</span> — follow suit if you can. Trump must be established before it can be led.</li>
                <li><span class="text-accent">Win</span> — Team 1 needs 6 + bet sets. Team 2 needs 8 &minus; bet sets.</li>
            </ul>
        </div>

        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-2">About</h2>
            <p class="text-muted-foreground">
                Built as a learning project. Open source on
                <a href="https://github.com/bedminer1/BRIDGE" target="_blank" class="text-accent hover:underline">
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
