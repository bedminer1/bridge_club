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
    let { matchRecords, message, username } = $state(data)

    $effect(() => { headerState.username = username ?? "" })
    $effect(() => { headerState.loggedIn = message === "success" })
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    {#if message !== "success"}
        <p class="text-lg text-muted-foreground mt-20">
            <a href="/login" class="text-accent hover:underline">{message}</a>
        </p>
    {:else}
        <!-- User header -->
        <div class="flex items-center gap-3 mb-8">
            <div class="w-10 h-10 rounded-full bg-accent/20 flex items-center justify-center text-accent font-bold text-lg">
                {username?.charAt(0).toUpperCase() ?? "?"}
            </div>
            <div>
                <h1 class="text-xl font-bold">{username}</h1>
                <p class="text-xs text-muted-foreground">{matchRecords.length} match{matchRecords.length !== 1 ? "es" : ""} played</p>
            </div>
        </div>

        <!-- Logout -->
        <form action="?/logout" method="POST" class="mb-10">
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
                                        <div class="flex gap-3 mb-6 text-xs text-muted-foreground">
                                            <span>{matchRecord.betSize}{matchRecord.trumpSuit.toUpperCase()}</span>
                                            <span>|</span>
                                            <span>{matchRecord.botDifficulty}</span>
                                            <span>|</span>
                                            <span>{formatDate(matchRecord.date)}</span>
                                        </div>
                                        <div class="mt-2 ml-4 flex">
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
