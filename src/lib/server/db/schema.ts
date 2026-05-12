import type { InferSelectModel } from 'drizzle-orm';
import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

export const users = sqliteTable('users', {
	id: integer('id').primaryKey(),
	username: text('username').notNull(),
	password: text('password').notNull(), // encrypted
});

export const matches = sqliteTable('matches', {
	id: integer("id").primaryKey(),

	// Metadata
	userID: integer("user_id").notNull(), // The human player
	date: integer("date").notNull(),      // Unix timestamp
	botDifficulty: text("bot_difficulty").notNull(),
	
	// Betting Info
	trumpSuit: text("trump_suit").notNull(),
	betSize: integer("bet_size").notNull(),
	betWinner: integer("bet_winner").notNull(), // 1-4
	
	// Match result
	partner: integer("partner"), // 2-4
	wonMatch: integer("won_match"), // 0 or 1

	// Sets won
	player1Sets: integer("player1_sets").notNull(),
	player2Sets: integer("player2_sets").notNull(),
	player3Sets: integer("player3_sets").notNull(),
	player4Sets: integer("player4_sets").notNull(),

	// Hands
	player1Hand: text("player1_hand").notNull(),
	player2Hand: text("player2_hand").notNull(),
	player3Hand: text("player3_hand").notNull(),
	player4Hand: text("player4_hand").notNull(),
})

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