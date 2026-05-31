import { fail, superValidate } from "sveltekit-superforms"
import { userFormSchema } from "../userSchema.js"
import { zod } from "sveltekit-superforms/adapters"
import { redirect } from "@sveltejs/kit"

const API_URL = process.env.API_URL || "http://127.0.0.1:3000"

export async function load() {
    return {
        form: await superValidate(zod(userFormSchema))
    }
}

export const actions = {
    default: async (event) => {
        const form = await superValidate(event, zod(userFormSchema))
        
        if (!form.valid) {
            return fail(400, { form })
        }
        const username = form.data.username
        const password = form.data.password as string

        // Call Rust backend
        const res = await fetch(`${API_URL}/api/auth/login`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ username, password }),
        })

        const data = await res.json()

        if (!data.ok) {
            form.errors.username = [data.error || "Login failed"]
            form.errors.password = [data.error || "Login failed"]
            return fail(400, { form })
        }

        // Set session cookie from backend response
        event.cookies.set("session", data.token, {
            path: "/",
            httpOnly: true,
            sameSite: "lax",
            expires: new Date(Date.now() + 1000 * 60 * 60 * 24 * 30),
        })

        redirect(307, "/")
    }
}
