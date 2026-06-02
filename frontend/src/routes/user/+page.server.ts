const API_URL = process.env.API_URL || "http://127.0.0.1:3000"

export async function load({ cookies }) {
    const token = cookies.get("session")

    if (!token) {
        return {
            matchRecords: [],
            message: "Log in to view stats",
            userStats: null,
            rank: 0
        }
    }

    // Validate session against Rust backend
    const sessionRes = await fetch(`${API_URL}/api/auth/session?token=${encodeURIComponent(token)}`)
    const sessionData = await sessionRes.json()

    if (!sessionData.ok) {
        return {
            matchRecords: [],
            message: "Session expired",
            userStats: null,
            rank: 0
        }
    }

    // Fetch matches
    const matchesRes = await fetch(`${API_URL}/api/matches`, {
        headers: { "X-Session-Token": token }
    })
    const matchesData = await matchesRes.json()

    return {
        matchRecords: matchesData.matches || [],
        message: "success",
        username: sessionData.user?.username || "",
        userID: sessionData.user?.id ?? 0,
        rank: 0,
        userStats: sessionData.user ? {
            gamesPlayed: sessionData.user.gamesPlayed,
            gamesWon: sessionData.user.gamesWon,
            totalSetsWon: sessionData.user.totalSetsWon,
            mostSetsWon: sessionData.user.mostSetsWon,
            elo: sessionData.user.elo ?? 500,
        } : null
    }
}

export const actions = {
    logout: async ({ cookies }) => {
        const token = cookies.get("session")
        if (token) {
            // Invalidate session on backend
            await fetch(`${API_URL}/api/auth/logout`, {
                method: "POST",
                headers: { "X-Session-Token": token },
            })
        }
        cookies.delete("session", { path: "/" })
    }
}
