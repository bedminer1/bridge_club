import { db } from '$lib/server/db/index.js'
import { matches } from '$lib/server/db/schema.js'
import { eq } from 'drizzle-orm'

const API_URL = process.env.API_URL || "http://127.0.0.1:3000"

export async function load({ params, cookies }) {
    const matchID = Number(params.matchID)

    const matchRecords: any[] = await db
        .select()
        .from(matches)
        .where(eq(matches.id, matchID))

    const matchRecord = matchRecords[0] || null

    // Get current user ID from session
    const token = cookies.get("session")
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
        matchRecord,
        userID
    }
}
