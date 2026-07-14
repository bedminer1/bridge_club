<script lang="ts">
    import { onMount } from "svelte"
    import { goto } from "$app/navigation"
    import { page } from "$app/state"
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
    import {
        getCachedMatchHistory,
        mergeMatchRecords,
        setCachedMatchHistory,
        USER_HISTORY_PAGE_SIZE,
        USER_HISTORY_REFRESH_LIMIT,
    } from "$lib/user-history-cache"

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
    let { message, username, userID, rank, userStats, token } = $state(data)

    let API_URL: string
    if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
        API_URL = "http://127.0.0.1:3000"
    } else {
        API_URL = "https://bridge-club.duckdns.org"
    }

    // Default stats to 0 when not logged in
    let s = $derived(userStats ?? { gamesPlayed: 0, gamesWon: 0, totalSetsWon: 0, mostSetsWon: 0, elo: 500 })

    let matchRecords = $state<any[]>(data.matchRecords ?? [])
    let historyLoading = $state(message === "success" && !!token)
    let historyLoadingMore = $state(false)
    let historyError = $state("")
    let historyHasMoreOlder = $state(false)
    let historyLatestId = $state<number | null>(null)
    let historyOldestId = $state<number | null>(null)

    function syncHistoryState(records: any[], hasMoreOlder: boolean) {
        matchRecords = records
        historyHasMoreOlder = hasMoreOlder
        historyLatestId = records.length > 0 ? Number(records[0].id) : null
        historyOldestId = records.length > 0 ? Number(records[records.length - 1].id) : null
    }

    async function fetchMatchHistory(query: URLSearchParams) {
        const res = await fetch(`${API_URL}/api/matches?${query.toString()}`, {
            headers: { "X-Session-Token": token },
        })

        if (!res.ok) {
            const errorData = await res.json().catch(() => ({}))
            throw new Error(errorData?.error || "Failed to load match history")
        }

        return res.json()
    }

    async function refreshMatchHistory() {
        if (message !== "success" || !token || !userID) {
            historyLoading = false
            return
        }

        historyError = ""
        const cached = getCachedMatchHistory(userID)

        if (cached) {
            syncHistoryState(cached.matchRecords, cached.hasMoreOlder)
            historyLatestId = cached.newestMatchId
            historyOldestId = cached.oldestMatchId
        }

        historyLoading = true

        try {
            if (!cached) {
                const query = new URLSearchParams({ limit: String(USER_HISTORY_PAGE_SIZE) })
                const data = await fetchMatchHistory(query)
                const records = Array.isArray(data.matches) ? data.matches : []
                syncHistoryState(records, Boolean(data.hasMoreOlder))
                setCachedMatchHistory(userID, matchRecords, historyHasMoreOlder)
                return
            }

            let nextAfterId = cached.newestMatchId
            if (nextAfterId === null) {
                historyLoading = false
                return
            }

            let hasMoreNewer = true
            while (hasMoreNewer) {
                const query = new URLSearchParams({
                    limit: String(USER_HISTORY_REFRESH_LIMIT),
                    afterId: String(nextAfterId),
                })
                const data = await fetchMatchHistory(query)
                const records = Array.isArray(data.matches) ? data.matches : []

                if (records.length === 0) {
                    break
                }

                syncHistoryState(mergeMatchRecords(matchRecords, records), historyHasMoreOlder)
                setCachedMatchHistory(userID, matchRecords, historyHasMoreOlder)

                nextAfterId = historyLatestId
                hasMoreNewer = Boolean(data.hasMoreNewer)
                if (!hasMoreNewer || nextAfterId === null) {
                    break
                }
            }
        } catch (error) {
            historyError = error instanceof Error ? error.message : "Failed to load match history"
        } finally {
            historyLoading = false
        }
    }

    async function loadOlderMatches() {
        if (!historyHasMoreOlder || historyLoadingMore || !historyOldestId || message !== "success" || !token || !userID) {
            return
        }

        historyLoadingMore = true
        historyError = ""

        try {
            const query = new URLSearchParams({
                limit: String(USER_HISTORY_PAGE_SIZE),
                beforeId: String(historyOldestId),
            })
            const data = await fetchMatchHistory(query)
            const records = Array.isArray(data.matches) ? data.matches : []
            const merged = mergeMatchRecords(matchRecords, records)
            syncHistoryState(merged, Boolean(data.hasMoreOlder))
            setCachedMatchHistory(userID, merged, historyHasMoreOlder)
        } catch (error) {
            historyError = error instanceof Error ? error.message : "Failed to load more matches"
        } finally {
            historyLoadingMore = false
        }
    }

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

    if (typeof window !== "undefined" && message === "success" && userID) {
        const cached = getCachedMatchHistory(userID)
        if (cached) {
            syncHistoryState(cached.matchRecords, cached.hasMoreOlder)
            historyLatestId = cached.newestMatchId
            historyOldestId = cached.oldestMatchId
        }
    }

    onMount(() => {
        void refreshMatchHistory()
    })

    // Settings state
    let showSettings = $state(false)
    // Show welcome + settings by default for new OAuth users
    let isNewUser = $state(typeof window !== 'undefined' && new URLSearchParams(window.location.search).get('new') === '1')
    $effect(() => {
        if (isNewUser) showSettings = true
    })
    // Change name
    let cnNew = $state("")
    let cnError = $state("")
    let cnSuccess = $state(false)
    let cnLoading = $state(false)

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
                body: JSON.stringify({ newUsername: cnNew }),
            })
            const data = await res.json()
            if (data.ok) {
                cnSuccess = true
                username = cnNew
                cnNew = ""
                // If new user coming from a room invite, redirect to the room
                if (isNewUser) {
                    try {
                        const pendingRoom = sessionStorage.getItem("bridgePendingRoom")
                        if (pendingRoom) {
                            sessionStorage.removeItem("bridgePendingRoom")
                            setTimeout(() => goto(`/?room=${encodeURIComponent(pendingRoom)}`), 800)
                        }
                    } catch {}
                }
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

        <!-- Welcome prompt for new OAuth users -->
        {#if isNewUser && !cnSuccess}
            <div class="w-full max-w-md mb-6 p-4 rounded-lg border border-accent/30 bg-accent/5 text-center">
                <p class="text-sm font-semibold text-accent mb-2">Welcome! Choose a username</p>
                <p class="text-xs text-muted-foreground mb-3">Your auto-generated username is <strong>{username}</strong>. Pick one you like below.</p>
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
                        {#if cnError}<p class="text-xs text-destructive">{cnError}</p>{/if}
                        {#if cnSuccess}<p class="text-xs text-green">Username changed!</p>{/if}
                        <Button onclick={changeName} disabled={cnLoading || !cnNew} size="sm" class="self-end">
                            {cnLoading ? "Saving..." : "Save"}
                        </Button>
                    </Card.CardContent>
                </Card.Root>
            </div>
        {/if}

        <!-- Theme toggle -->

        <!-- Match history -->
        <div class="w-full max-w-3xl">
            <div class="mb-4 flex items-center justify-between gap-3">
                <h2 class="text-sm font-semibold text-muted-foreground uppercase tracking-wider">Match History</h2>
                {#if historyLoading && matchRecords.length > 0}
                    <span class="flex items-center gap-2 text-xs text-muted-foreground">
                        <span class="h-3.5 w-3.5 rounded-full border-2 border-muted-foreground/30 border-t-accent animate-spin"></span>
                        Syncing history
                    </span>
                {/if}
            </div>

            {#if historyError}
                <p class="mb-3 text-xs text-destructive">{historyError}</p>
            {/if}

            {#if historyLoading && matchRecords.length === 0}
                <div class="flex items-center gap-2 text-sm text-muted-foreground py-6">
                    <span class="h-4 w-4 rounded-full border-2 border-muted-foreground/30 border-t-accent animate-spin"></span>
                    <span>Loading match history...</span>
                </div>
            {:else if matchRecords.length > 0}
                <div class="flex flex-col gap-3 mb-20">
                    {#each matchRecords as matchRecord}
                        {@const { didWin, eloChange } = matchResult(matchRecord)}
                        <a href="/user/{matchRecord.id}" class="block">
                            <Card.Root class="w-full rounded-lg border-border hover:border-accent/30 transition-colors">
                                <div class="flex items-center justify-between p-4 gap-4">
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

                                    <div class="hidden sm:block shrink-0">
                                        <ScoreDisplay {matchRecord} />
                                    </div>
                                </div>
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

                    {#if historyHasMoreOlder}
                        <Button
                            onclick={loadOlderMatches}
                            disabled={historyLoadingMore}
                            variant="outline"
                            class="self-center mt-2"
                        >
                            {historyLoadingMore ? "Loading older matches..." : "Load older matches"}
                        </Button>
                    {/if}
                </div>
            {:else}
                <p class="text-sm text-muted-foreground">No matches played yet.</p>
            {/if}
        </div>
    {/if}
</div>
