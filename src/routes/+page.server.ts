import { db } from '$lib/server/db/index.js'
import { users } from "$lib/server/db/schema.js"
import { eq, and } from 'drizzle-orm';

export const actions = {
    default: async ({ request }) => {
        const data = await request.formData()
        const username = data.get('username')
        const password = data.get('password')

        if (typeof username !== 'string' || typeof password !== 'string') {
            throw new Error('Invalid form data')
        }

        // fetch from users, check if exists, get userID
        const user = await db
            .select()
            .from(users)
            .where(and(eq(users.username, username)))

        console.log(user)
    }
}