import { fail, superValidate } from "sveltekit-superforms"
import { userFormSchema } from "../userSchema.js"
import { zod } from "sveltekit-superforms/adapters"
import { redirect } from "@sveltejs/kit"
import { db } from "$lib/server/db/index.js"
import { users } from "$lib/server/db/schema.js"
import { eq } from "drizzle-orm"
import { hashPassword } from "$lib/server/crypto.js"
import { generateSessionToken, createSession } from "$lib/auth.js"

export async function load() {
    return {
        form: await superValidate(zod(userFormSchema))
    }
}

export const actions = {
    default: async (event) => {
        const form = await superValidate(event, zod(userFormSchema))
        
        if (!form.valid) {
            return fail(400, {
                form,
            })
        }
        const username = form.data.username
        const password = form.data.password

        // check if already exists
        const existingUsers = await db
            .select()
            .from(users)
            .where(eq(users.username, username))

        if (existingUsers.length > 0) {
            form.errors.username = ["Username already used"]
            console.log(form)
            return fail(400, {
                form
            })
        }

        const hashedPassword = await hashPassword(password)
        const insertedUsers = await db.insert(users).values({
            username,
            password: hashedPassword
        }).returning({ insertedID: users.id })

        const user = existingUsers[0]
                const token = generateSessionToken()
                const session = createSession(token, insertedUsers[0].insertedID)
                event.cookies.set("session", token, { path: "/", httpOnly: true, sameSite: "lax", expires: (await session).expiresAt})

        redirect(307, "/")
    }
}