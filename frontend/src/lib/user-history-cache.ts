const MEMORY_CACHE = new Map<number, MatchHistoryCache>()
const STORAGE_PREFIX = "bridge-user-history:"

export const USER_HISTORY_PAGE_SIZE = 20
export const USER_HISTORY_REFRESH_LIMIT = 100

export type MatchHistoryCache = {
    matchRecords: any[]
    hasMoreOlder: boolean
    newestMatchId: number | null
    oldestMatchId: number | null
    updatedAt: number
}

function storageKey(userId: number): string {
    return `${STORAGE_PREFIX}${userId}`
}

function toNumberOrNull(value: unknown): number | null {
    return typeof value === "number" && Number.isFinite(value) ? value : null
}

export function mergeMatchRecords(existing: any[], incoming: any[]): any[] {
    const map = new Map<number, any>()

    for (const record of [...existing, ...incoming]) {
        const id = Number(record?.id)
        if (Number.isFinite(id)) {
            map.set(id, record)
        }
    }

    return [...map.values()].sort((a, b) => Number(b.id) - Number(a.id))
}

export function getCachedMatchHistory(userId: number): MatchHistoryCache | null {
    const memoryCached = MEMORY_CACHE.get(userId)
    if (memoryCached) {
        return memoryCached
    }

    if (typeof window === "undefined") {
        return null
    }

    try {
        const raw = localStorage.getItem(storageKey(userId))
        if (!raw) {
            return null
        }

        const parsed = JSON.parse(raw) as Partial<MatchHistoryCache>
        const matchRecords = Array.isArray(parsed.matchRecords) ? parsed.matchRecords : []
        const cached: MatchHistoryCache = {
            matchRecords: mergeMatchRecords([], matchRecords),
            hasMoreOlder: Boolean(parsed.hasMoreOlder),
            newestMatchId: toNumberOrNull(parsed.newestMatchId),
            oldestMatchId: toNumberOrNull(parsed.oldestMatchId),
            updatedAt: typeof parsed.updatedAt === "number" ? parsed.updatedAt : Date.now(),
        }

        MEMORY_CACHE.set(userId, cached)
        return cached
    } catch {
        return null
    }
}

export function setCachedMatchHistory(userId: number, matchRecords: any[], hasMoreOlder: boolean): MatchHistoryCache {
    const sortedRecords = mergeMatchRecords([], matchRecords)
    const cached: MatchHistoryCache = {
        matchRecords: sortedRecords,
        hasMoreOlder,
        newestMatchId: toNumberOrNull(sortedRecords[0]?.id),
        oldestMatchId: toNumberOrNull(sortedRecords[sortedRecords.length - 1]?.id),
        updatedAt: Date.now(),
    }

    MEMORY_CACHE.set(userId, cached)

    if (typeof window !== "undefined") {
        try {
            localStorage.setItem(storageKey(userId), JSON.stringify(cached))
        } catch {
        }
    }

    return cached
}

export function clearCachedMatchHistory(userId: number): void {
    MEMORY_CACHE.delete(userId)

    if (typeof window !== "undefined") {
        try {
            localStorage.removeItem(storageKey(userId))
        } catch {
        }
    }
}
