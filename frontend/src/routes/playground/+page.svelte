<script lang="ts">
    import { Gamepad2, Crown, Github } from "@lucide/svelte"
    import { suitToSymbol } from "$lib/utils"

    const suits = { c: "Club", d: "Diamond", h: "Heart", s: "Spades" }
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-2">Playground</h1>
    <p class="text-sm text-muted-foreground mb-8">Please ignore this page, it's just for testing.</p>

    <div class="w-full max-w-lg flex flex-col gap-4 text-sm">
        <!-- How to Play -->
        <div class="rounded-lg border border-border bg-card p-4">
            
        </div>

        <!-- About -->
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-2">About</h2>
            <p class="text-muted-foreground">
                Built as a learning project. Open source on
                <a href="https://github.com/bedminer1/bridge_club" target="_blank" class="text-accent hover:underline">
                    GitHub <Github class="inline w-3 h-3" />
                </a>.
            </p>
        </div>

        <!-- Links -->
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
