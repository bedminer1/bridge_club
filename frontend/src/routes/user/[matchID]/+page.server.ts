import { db } from '$lib/server/db/index.js'
import { matches } from '$lib/server/db/schema.js'
import { eq } from 'drizzle-orm'

export async function load({ params }) {
    const matchID = Number(params.matchID)

    const matchRecords: MatchRecord[] = await db
        .select()
        .from(matches)
        .where(eq(matches.id, matchID))

    const matchRecord = matchRecords[0]

    return {
        matchRecord
    }
}