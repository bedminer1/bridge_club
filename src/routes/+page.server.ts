import { db } from '$lib/server/db/index.js'
import { users, matches } from "$lib/server/db/schema.js"
import { eq } from 'drizzle-orm';
import { hashPassword } from '$lib/server/crypto.js';
import { invalidateAllSessions, validateSessionToken } from '$lib/auth.js';

export async function load({ cookies }) {
    // validate session cookie
    const token = cookies.get("session")
    if (token === undefined) {
        console.log("error: Cookie not found")
        return {
            userID: 0
        }
    }
    
    const { session, user } = await validateSessionToken(token)

    if (!session) {
        console.log("Session invalid")
        return {
            userID: 0
        } 
    }
    if (!user) {
        console.log("User not found")
        return {
            userID: 0
        }
    }

    // console.log(
    //     "Username: ", user.username,
    //     "\nUserID: ", user.id
    // )

    return {
        userID: session.userID,
        username: user.username
    }
}

export const actions = {
    saveMatch: async ({ request }) => {
        const data = await request.formData()
        const userID = Number(data.get("userID"))
        const date = Number(data.get('date'))
        const botDifficulty = String(data.get('botDifficulty'))

        const trumpSuit = String(data.get('trumpSuit'))
        const betSize = Number(data.get('betSize'))
        const betWinner = Number(data.get('betWinner'))

        const partner = Number(data.get('partner'))
        const wonMatch = Number(data.get('wonMatch'))

        const player1Sets = Number(data.get('player1Sets'))
        const player2Sets = Number(data.get('player2Sets'))
        const player3Sets = Number(data.get('player3Sets'))
        const player4Sets = Number(data.get('player4Sets'))

        const player1Hand = String(data.get('player1Hand'))
        const player2Hand = String(data.get('player2Hand'))
        const player3Hand = String(data.get('player3Hand'))
        const player4Hand = String(data.get('player4Hand'))
        
        await db.insert(matches)
                .values({
                    userID,
                    date,
                    botDifficulty,
                    trumpSuit,
                    betSize,
                    betWinner,
                    partner,
                    wonMatch,
                    player1Sets,
                    player2Sets,
                    player3Sets,
                    player4Sets,
                    player1Hand,
                    player2Hand,
                    player3Hand,
                    player4Hand
                })
    },

    logout: async ({ request, cookies }) => {
        const data = await request.formData()
        const userIDStr = data.get("userID")
        if (userIDStr === null) return

        const userID = Number(userIDStr)
        if (isNaN(userID)) return

        cookies.delete("sessions", { path: "/", httpOnly: true, sameSite: "lax"})

        invalidateAllSessions(Number(userID))
    }
}