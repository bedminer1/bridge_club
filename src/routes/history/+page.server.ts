import { db } from "$lib/server/db"
import { matches } from "$lib/server/db/schema"
import { validateSessionToken } from "$lib/auth"
import { eq } from "drizzle-orm"

export async function load({ cookies }) {
    const token = cookies.get("session")
    if (token === undefined) {
            console.log("error: Cookie not found")
            return {
                matchRecords: []
            }
        }
        
    const { session, user } = await validateSessionToken(token)
    if (!session) {
        console.log("Session invalid")
        return {
            matchRecords: []
        } 
    }
    if (!user) {
        console.log("User not found")
        return {
            matchRecords: []
        }
    }

    const matchRecords: MatchRecord[] = await db
        .select()
        .from(matches)
        .where(eq(matches.userID, user.id))
    
    return {
        matchRecords : matchRecords.reverse()
    }
}