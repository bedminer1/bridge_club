import { fail, superValidate } from "sveltekit-superforms"
import { userFormSchema } from "../userSchema.js"
import { zod } from "sveltekit-superforms/adapters"
import { redirect } from "@sveltejs/kit"
import { db } from "$lib/server/db/index.js"
import { users } from "$lib/server/db/schema.js"
import { eq, and } from "drizzle-orm"
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
        const hashedPassword = await hashPassword(password as string)

        // check if exists
        const existingUsers = await db
            .select()
            .from(users)
            .where(and(eq(users.username, username), eq(users.password, hashedPassword)))

        if (existingUsers.length < 1) {
            form.errors.username = ["Username or password incorrect"]
            form.errors.password = ["Username or password incorrect"]

            return fail(400, {
                form
            })
        }

        const user = existingUsers[0]
        const token = generateSessionToken()
        const session = createSession(token, user.id)
        event.cookies.set("session", token, { path: "/", httpOnly: true, sameSite: "lax", expires: (await session).expiresAt})

        redirect(307, "/")
    }
}