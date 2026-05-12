import { fail, superValidate } from "sveltekit-superforms"
import { signupFormSchema } from "./signupSchema.js"
import { zod } from "sveltekit-superforms/adapters"
import { redirect } from "@sveltejs/kit"
import { db } from "$lib/server/db/index.js"
import { users } from "$lib/server/db/schema.js"
import { eq } from "drizzle-orm"
import { hashPassword } from "$lib/server/crypto.js"

export async function load() {
    return {
        form: await superValidate(zod(signupFormSchema))
    }
}

export const actions = {
    default: async (event) => {
        const form = await superValidate(event, zod(signupFormSchema))
        
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
        await db.insert(users).values({
            username,
            password: hashedPassword
        })

        redirect(307, "/")
    }
}