const API_URL = process.env.API_URL || "http://127.0.0.1:3000"

export async function load({ params, cookies }) {
    const matchID = Number(params.matchID)
    const token = cookies.get("session")

    // Get match from Rust API
    const res = await fetch(`${API_URL}/api/matches/${matchID}`)
    const data = await res.json()

    // Get current user ID from session
    let userID = 0
    if (token) {
        try {
            const sessionRes = await fetch(`${API_URL}/api/auth/session?token=${encodeURIComponent(token)}`)
            const sessionData = await sessionRes.json()
            if (sessionData.ok && sessionData.user?.id) {
                userID = sessionData.user.id
            }
        } catch {}
    }

    return {
        matchRecord: data.ok ? data.match : null,
        userID
    }
}
