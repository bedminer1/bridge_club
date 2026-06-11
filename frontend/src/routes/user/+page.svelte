<script lang="ts">
    import ScoreDisplay from "./ScoreDisplay.svelte";
    import PokerCard from "$lib/components/poker-card.svelte";
    import * as Card from "$lib/components/ui/card/index"
    import { formatDate } from "$lib/utils";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { headerState } from "$lib/game/header-state.svelte";
    import { toggleMode } from "mode-watcher";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    /** Parse a compact preview string like "2cw3hlAdw..." into Card objects. */
    function parsePreview(preview: string): Card[] {
        const suitMap: Record<string, string> = { c: "Club", d: "Diamond", h: "Heart", s: "Spades" }
        const rankToValue: Record<string, number> = {
            "2": 2, "3": 3, "4": 4, "5": 5, "6": 6, "7": 7, "8": 8, "9": 9,
            "10": 10, "J": 11, "Q": 12, "K": 13, "A": 14,
        }
        const valueToRank: Record<number, string> = {
            2: "2", 3: "3", 4: "4", 5: "5", 6: "6", 7: "7", 8: "8", 9: "9",
            10: "10", 11: "J", 12: "Q", 13: "K", 14: "A",
        }
        const cards: Card[] = []
        let i = 0
        while (i < preview.length) {
            const isTen = preview[i] === "1" && preview[i + 1] === "0"
            const rankStr = isTen ? "10" : preview[i]
            const suitLetter = preview[isTen ? i + 2 : i + 1]
            const won = preview[isTen ? i + 3 : i + 2] === "w"
            const value = rankToValue[rankStr]
            if (value && suitLetter) {
                cards.push({
                    Rank: valueToRank[value] ?? rankStr,
                    Value: value,
                    Suit: suitMap[suitLetter] ?? "Club",
                    WonSet: won,
                })
            }
            i += isTen ? 4 : 3
        }
        return cards
    }

    let { data } = $props()
    let { matchRecords, message, username, userID, rank, userStats, token } = $state(data)

    let API_URL: string
    if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
        API_URL = "http://127.0.0.1:3000"
    } else {
        API_URL = "https://bridge-club.duckdns.org"
    }

    // Default stats to 0 when not logged in
    let s = $derived(userStats ?? { gamesPlayed: 0, gamesWon: 0, totalSetsWon: 0, mostSetsWon: 0, elo: 500 })

    // Determine match result for the current user
    let matchResult = $derived.by(() => {
        return (m: any) => {
            const myParticipant = m.participants?.find((p: any) => p.userId === userID)
            if (myParticipant) {
                const didWin = myParticipant.team === m.winningTeam
                return { didWin, eloChange: myParticipant.eloChange ?? 0 }
            }
            return { didWin: false, eloChange: 0 }
        }
    })

    // Settings state
    let showSettings = $state(false)
    // Change password
    let cpCurrent = $state("")
    let cpNew = $state("")
    let cpError = $state("")
    let cpSuccess = $state(false)
    let cpLoading = $state(false)
    // Change name
    let cnPassword = $state("")
    let cnNew = $state("")
    let cnError = $state("")
    let cnSuccess = $state(false)
    let cnLoading = $state(false)

    async function changePassword() {
        cpError = ""
        cpSuccess = false
        if (!cpCurrent || !cpNew || cpNew.length < 6) {
            cpError = "New password must be at least 6 characters"
            return
        }
        cpLoading = true
        try {
            const res = await fetch(`${API_URL}/api/auth/change-password`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": token },
                body: JSON.stringify({ currentPassword: cpCurrent, newPassword: cpNew }),
            })
            const data = await res.json()
            if (data.ok) {
                cpSuccess = true
                cpCurrent = ""
                cpNew = ""
            } else {
                cpError = data.error || "Failed to change password"
            }
        } catch {
            cpError = "Connection error"
        }
        cpLoading = false
    }

    async function changeName() {
        cnError = ""
        cnSuccess = false
        if (!cnNew || cnNew.length < 2) {
            cnError = "Username must be at least 2 characters"
            return
        }
        cnLoading = true
        try {
            const res = await fetch(`${API_URL}/api/auth/change-name`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Session-Token": token },
                body: JSON.stringify({ password: cnPassword, newUsername: cnNew }),
            })
            const data = await res.json()
            if (data.ok) {
                cnSuccess = true
                username = cnNew
                cnPassword = ""
                cnNew = ""
            } else {
                cnError = data.error || "Failed to change username"
            }
        } catch {
            cnError = "Connection error"
        }
        cnLoading = false
    }

    // Re-fetch user stats on mount
    $effect(() => {
        if (message === "success" && token) {
            fetch(`${API_URL}/api/auth/session?token=${encodeURIComponent(token)}`)
                .then(r => r.json())
                .then(d => {
                    if (d.ok && d.user) {
            userStats = {
                gamesPlayed: d.user.gamesPlayed ?? 0,
                gamesWon: d.user.gamesWon ?? 0,
                totalSetsWon: d.user.totalSetsWon ?? 0,
                mostSetsWon: d.user.mostSetsWon ?? 0,
                elo: d.user.elo ?? 500,
            }
                    }
                })
                .catch(() => {})
        }
    })

    /** Get the preview cards for the current user from their participant entry */
    function getUserPreview(matchRecord: any): Card[] {
        if (!matchRecord.participants) return []
        const myP = matchRecord.participants.find((p: any) => p.userId === userID)
        if (myP?.handPreview) {
            return parsePreview(myP.handPreview)
        }
        const firstP = matchRecord.participants[0]
        if (firstP?.handPreview) return parsePreview(firstP.handPreview)
        return []
    }

    $effect(() => { headerState.username = username ?? "" })
    $effect(() => { headerState.loggedIn = message === "success" })

    let winrate = $derived(s.gamesPlayed > 0 ? ((s.gamesWon / s.gamesPlayed) * 100).toFixed(1) : "0.0")
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
                    <span><strong class="text-accent">{s.elo}</strong> elo</span>
                </div>
                <Separator class="mt-3" />
            </div>
        {/if}

        <!-- Logout + Settings toggle -->
        <div class="flex items-center gap-4 mb-4">
            <form action="?/logout" method="POST">
                <button class="text-xs text-muted-foreground hover:text-destructive transition-colors underline underline-offset-2">
                    logout
                </button>
            </form>
            <button onclick={() => showSettings = !showSettings} class="text-xs text-accent hover:underline underline-offset-2">
                {showSettings ? "Hide Settings" : "Account Settings"}
            </button>
        </div>

        <!-- Theme toggle -->
        <div class="flex items-center gap-3 mb-6">
            <span class="text-xs text-muted-foreground">Light Mode</span>
            <Switch bind:checked={headerState.isLightMode} onclick={toggleMode} />
        </div>

        <!-- Account Settings -->
        {#if showSettings && token}
            <div class="w-full max-w-md mb-8 flex flex-col gap-4">
                <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wider">Account Settings</h2>

                <!-- Change Username -->
                <Card.Root>
                    <Card.CardHeader>
                        <Card.CardTitle class="text-sm">Change Username</Card.CardTitle>
                    </Card.CardHeader>
                    <Card.CardContent class="flex flex-col gap-3">
                        <Input bind:value={cnNew} placeholder="New username" disabled={cnLoading} />
                        <Input bind:value={cnPassword} type="password" placeholder="Current password" disabled={cnLoading} />
                        {#if cnError}<p class="text-xs text-destructive">{cnError}</p>{/if}
                        {#if cnSuccess}<p class="text-xs text-green">Username changed!</p>{/if}
                        <Button onclick={changeName} disabled={cnLoading || !cnNew || !cnPassword} size="sm" class="self-end">
                            {cnLoading ? "Saving..." : "Save"}
                        </Button>
                    </Card.CardContent>
                </Card.Root>

                <!-- Change Password -->
                <Card.Root>
                    <Card.CardHeader>
                        <Card.CardTitle class="text-sm">Change Password</Card.CardTitle>
                    </Card.CardHeader>
                    <Card.CardContent class="flex flex-col gap-3">
                        <Input bind:value={cpCurrent} type="password" placeholder="Current password" disabled={cpLoading} />
                        <Input bind:value={cpNew} type="password" placeholder="New password (min 6 chars)" disabled={cpLoading} />
                        {#if cpError}<p class="text-xs text-destructive">{cpError}</p>{/if}
                        {#if cpSuccess}<p class="text-xs text-green">Password changed!</p>{/if}
                        <Button onclick={changePassword} disabled={cpLoading || !cpCurrent || !cpNew} size="sm" class="self-end">
                            {cpLoading ? "Saving..." : "Save"}
                        </Button>
                    </Card.CardContent>
                </Card.Root>
            </div>
        {/if}

        <!-- Match history -->
        <div class="w-full max-w-3xl">
            <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-4">Match History</h2>

            {#if matchRecords.length > 0}
                <div class="flex flex-col gap-3">
                            {#each matchRecords as matchRecord}
                                {@const { didWin, eloChange } = matchResult(matchRecord)}
                                <a href="/user/{matchRecord.id}" class="block">
                                    <Card.Root class="w-full rounded-lg border-border hover:border-accent/30 transition-colors">
                                        <div class="flex items-center justify-between p-4 gap-4">
                                            <!-- Info + Result badge inline -->
                                            <div class="flex items-center gap-3 flex-1 min-w-0">
                                                <span class="text-sm font-bold shrink-0 {didWin ? 'text-blue' : 'text-red'}">
                                                    {didWin ? "Win" : "Loss"}
                                                </span>
                                                <span class="flex gap-3 text-xs text-muted-foreground flex-wrap items-center">
                                                    <span>{matchRecord.betSize}{matchRecord.trumpSuit.toUpperCase()}</span>
                                                    <span>|</span>
                                                    {#if matchRecord.isHidden}
                                                        <span class="text-purple font-semibold">Hidden Only</span>
                                                    {:else}
                                                        <span class="text-muted-foreground">Open</span>
                                                    {/if}
                                                    <span>|</span>
                                                    <span>{formatDate(matchRecord.createdAt)}</span>
                                                    {#if eloChange !== 0}
                                                        <span>|</span>
                                                        <span class="{eloChange > 0 ? 'text-green' : 'text-red'}">{eloChange > 0 ? '+' : ''}{eloChange}</span>
                                                    {/if}
                                                </span>
                                            </div>

                                            <!-- Scores -->
                                            <div class="hidden sm:block shrink-0">
                                                <ScoreDisplay {matchRecord} />
                                            </div>
                                        </div>
                                        <!-- Hand preview (from current user's participant entry) -->
                                        {@const previewCards = getUserPreview(matchRecord)}
                                        {#if previewCards.length > 0}
                                            <div class="px-4 pb-2 flex flex-wrap gap-0.5">
                                                {#each previewCards as card, ci (ci)}
                                                    <PokerCard card={card} isIllegal={false} minify={true} />
                                                {/each}
                                            </div>
                                        {/if}
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
