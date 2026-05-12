import { db } from '$lib/server/db/index.js'
import { users, matches } from "$lib/server/db/schema.js"
import { eq } from 'drizzle-orm';
import { hashPassword } from '$lib/server/crypto.js';

export const actions = {
    default: async ({ request }) => {
        const data = await request.formData()
        const username = data.get('username')
        const password = data.get('password')
        const hashedPassword = await hashPassword(password as string)

        if (typeof username !== 'string' || typeof password !== 'string') {
            throw new Error('Invalid username or password')
        }

        // fetch from users, check if exists, get userID
        const user = (await db
            .select()
            .from(users)
            .where(eq(users.username, username)))[0]

        if (!user) {
            console.log('User not found')
            return
        }
        const isValid = hashedPassword === user.password
        if (!isValid) {
            console.log('Incorrect password', user.password, hashedPassword)
            return
        }

        const date = Number(data.get('date'))

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
                    userID: user.id,
                    date,
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
    }
}