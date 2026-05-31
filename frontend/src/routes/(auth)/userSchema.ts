import { z } from "zod"

export const userFormSchema = z.object({
    username: z.string().min(2).max(50),
    password: z.string().min(2).max(50),
})

export type UserFormSchema = typeof userFormSchema