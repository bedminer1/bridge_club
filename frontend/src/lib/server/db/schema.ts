import type { InferSelectModel } from 'drizzle-orm';
import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

export const users = sqliteTable('users', {
	id: integer('id').primaryKey(),
	username: text('username').notNull(),
	password: text('password').notNull(), // encrypted
});

export const sessions = sqliteTable("sessions", {
	id: text("id").primaryKey(),
	userID: integer("user_id")
		.notNull()
		.references(() => users.id),
	expiresAt: integer("expires_at", {
		mode: "timestamp"
	}).notNull()
})

export type User = InferSelectModel<typeof users>
export type Session = InferSelectModel<typeof sessions>
