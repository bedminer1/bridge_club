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
