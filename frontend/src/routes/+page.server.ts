const API_URL = process.env.API_URL || "http://127.0.0.1:3000"

export async function load({ cookies }) {
    const token = cookies.get("session")
    let userID = 0
    let username = ""
    let sessionToken = ""

    if (token) {
        // Validate session against Rust backend
        const res = await fetch(`${API_URL}/api/auth/session?token=${encodeURIComponent(token)}`)
        const data = await res.json()
        if (data.ok && data.user) {
            userID = data.user.id
            username = data.user.username
            sessionToken = token
        }
    }

    return { userID, username, token: sessionToken }
}

export const actions = {
    saveMatch: async ({ cookies, request }) => {
        const token = cookies.get("session")
        if (!token) {
            return { ok: false, error: "Not logged in" }
        }

        const formData = await request.formData()

        // Build the match payload matching SaveMatchRequest
        const body = {
            date: Number(formData.get("date")),
            botDifficulty: formData.get("botDifficulty"),
            trumpSuit: formData.get("trumpSuit"),
            betSize: Number(formData.get("betSize")),
            betWinner: Number(formData.get("betWinner")),
            partner: formData.get("partner") ? Number(formData.get("partner")) : null,
            wonMatch: formData.get("wonMatch") !== null ? Number(formData.get("wonMatch")) : null,
            player1Sets: Number(formData.get("player1Sets")),
            player2Sets: Number(formData.get("player2Sets")),
            player3Sets: Number(formData.get("player3Sets")),
            player4Sets: Number(formData.get("player4Sets")),
            player1Hand: formData.get("player1Hand"),
            player2Hand: formData.get("player2Hand"),
            player3Hand: formData.get("player3Hand"),
            player4Hand: formData.get("player4Hand"),
        }

        const res = await fetch(`${API_URL}/api/matches`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "X-Session-Token": token,
            },
            body: JSON.stringify(body),
        })

        const data = await res.json()
        return { ok: data.ok }
    }
}
