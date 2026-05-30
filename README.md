# Bridge Club

https://bridge-club.vercel.app/

## Team Information

- **Team name:** Bridge Club
- **Proposed level of achievement:** Apollo

## Abstract

Bridge Club is a platform to play Singaporean Bridge. It offers both single-player and multiplayer with an aesthetic UI and game analytics. The backend comprises a core game engine, a server that enforces rules and runs bot players, a Svelte frontend, and a CLI client for quick testing. Current features support full dealing, bidding, partner selection, trick-taking play, scoring, and basic bot AI (Easy / Medium).

## Motivation

We really like Bridge — it combines the features of many other popular games.
- The simple game mechanics of taking turns to play cards like Uno.
- The strategy can go deeper with card counting like in Blackjack, and expected value calculations like in Poker.
- Mind games and bluffing like in Mafia and Among Us where you trick others about your team status.

We want an approachable, well-tested implementation of Singaporean Bridge that supports single-player play against bots and multiplayer play over the network, with analytics to help players improve. 

## Current Features

### Game Lobby
Create or join a lobby to play with your friends. If there aren't enough players, the remaining players will be replaced with bots. 

![Local Image](/frontend/static/milestone_1/game_lobby.png)

## Normal Gameplay

![Local Image](/frontend/static/milestone_1/normal_mode.png)

## Tutorial Mode
If you're new to the game, you can enable tutorial mode which will show you the hands of all the players.

![Local Image](/frontend/static/milestone_1/tutorial_mode.png)

## User Profile
View your statistics like Elo rating and your match history.

![Local Image](/frontend/static/milestone_1/profile.png)

## Leaderboard
See how you fare compared to everyone.

![Local Image](/frontend/static/milestone_1/leaderboard.png)

## Planned Features

- Enhanced bot difficulty tuning, randomized behavior to reduce predictability, 
- Improved tutorial mode with hints and guides.
- Matchmaking 
- Progress tracker and analytics: per-player stats, hand history, heatmaps of bidding/plays, and improvement suggestions.
- Improved UI: here we aim for a clean, simple aesthetic, focusing on functionality first. Once the features have been finalised, we are aiming for a more cosy, cute style similar to games like [Stardew Valley](https://www.stardewvalley.net/).

## Tech Stack

- Backend: Rust
- Frontend: Svelte + Vite (TypeScript)
- UI: shadcn-svelte
- Database: Turso (SQLite)
- Testing: C++ 
- Build & tooling: Cargo for Rust backend, npm for Svelte frontend
- Deployment: Vercel (Frontend), 

## Architecture

The project is built around a server-authoritative game core. The Rust backend owns the rules and state transitions, the Svelte frontend renders the player experience, and the C++ CLI is a lightweight utility client for local testing. Here is a truncated skeleton of our project structure.

```text
bridge_club/
├── backend/
│   ├── Cargo.toml
│   └── crates/
│       ├── game-core/
│       │   ├── src/lib.rs
│       │   ├── src/game.rs
│       │   ├── src/bid.rs
│       │   ├── src/card.rs
│       │   ├── src/deck.rs
│       │   ├── src/player.rs
│       │   └── src/scoring.rs
│       └── server/
│           ├── src/main.rs
│           ├── src/lib.rs
│           ├── src/routes.rs
│           ├── src/game_session.rs
│           ├── src/session.rs
│           ├── src/auth.rs
│           ├── src/db.rs
│           └── src/bot.rs
├── frontend/
│   ├── package.json
│   ├── src/
│   │   ├── app.css
│   │   ├── app.html
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   ├── game/
│   │   │   └── server/
│   │   └── routes/
│   │       ├── +layout.svelte
│   │       ├── +page.svelte
│   │       ├── (auth)/
│   │       ├── about/
│   │       ├── leaderboard/
│   │       └── user/
│   │           └── [matchID]/
│   └── static/
└── cli_interface/
    ├── main.cpp
    ├── Makefile
    ├── include/
    └── src/
```

Some important modules and files are listed below:

- `backend/crates/game-core` keeps the core bridge rules in one place.
	- `src/lib.rs` re-exports the public game types.
	- `src/game.rs` owns the `Table` state machine, phase transitions, legal actions, and win conditions.
	- `src/bid.rs`, `src/card.rs`, `src/deck.rs`, `src/player.rs`, and `src/scoring.rs` define the core bridge domain types.
- `backend/crates/server` turns the game core into an application.
	- `src/main.rs` boots the Axum server, loads env vars, and runs migrations.
	- `src/routes.rs` defines the HTTP API for auth, rooms, single-player games, match history, and the leaderboard.
	- `src/game_session.rs` coordinates single-player sessions and bot turns.
	- `src/session.rs` holds in-memory rooms, seated players, and shared app state.
	- `src/auth.rs` handles password hashing and session token creation/validation.
	- `src/db.rs` wraps Turso/libSQL access and schema migrations.
	- `src/bot.rs` contains bot bidding, partner selection, and play logic.
- `frontend` is the Svelte UI layer.
	- `src/routes/` contains page routes for the main game, auth flows, about page, leaderboard, and user pages.
	- `src/lib/components/` holds reusable UI components such as cards, headers, and hand displays.
	- `src/lib/game/` contains client-side game helpers and visual-state logic.
	- `src/lib/server/` contains server-side helpers used by the SvelteKit app.
	- `src/app.css`, `src/app.html`, and `src/routes/+layout.svelte` define the global look and shared layout.
- `cli_interface` is the native bridge client used for quick manual testing.
	- `main.cpp` is the entry point that deals cards and prints hands.
	- `include/` and `src/` hold the card, deck, player, and game implementation.
	- `Makefile` builds the CLI binary.

## Timeline
### Completed
| Week | Tasks |
| ---- | ----- |
| 0    | High-level architecture, spec out main features  |
| 1    | Initialise repository, SvelteKit + ShadCN setup  | 
| 2    | Rust backend setup, initialise game-core     | 
| 3    | Core game logic, database and match saving | 
| 4    | Authentication, testing, bug-fixes     | 

### Upcoming
| Week | Tasks |
| ---- | ----- |
| 5    |      | 
| 6    |      | 
| 7    |      | 


Detailed day-by-day tasks can be found in our project log. 
