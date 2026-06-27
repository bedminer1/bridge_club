# Bridge Club

https://bridge-club.duckdns.org/

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
During each trick, select the card you want to play. The previous trick is shown on the top right. 

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

### Polling → WebSockets

| Factor | Polling | WebSockets |
|--------|---------|------------|
| Implementation | 10 lines of setInterval | Connection mgmt, reconnection, state sync |
| Debugging | curl any endpoint, see JSON | Need WebSocket client, stateful |
| Server restart | Client gets 404, shows error | Client hangs until timeout |
| Auth | Stateless (token per request) | Need to auth on connect |
| Traffic | 2 req/s per client (1M reads → ~5 clients) | 1 connection per client, zero overhead when idle |


It makes sense to migrate to WebSockets because:

1. **Bot animation is janky.** Each bot turn requires: poll `/advance` → wait response → poll `/state` → re-render. With 3 bot turns between human actions, that's 6 HTTP round trips (12s at 2s intervals).
2. **Latency for multiplayer real humans.** If 4 humans play, a player's card play should show on everyone's screen in <200ms, not 2s later.
3. **Mobile browsers throttle setInterval.** iOS Safari kills background tabs' intervals → stale game state on return.

**Migration Phase 1 — WebSocket per room (not per player):**
```
Client connects: ws://host/ws?id={room_id}&token={session}
Server pushes:   {"type":"bot_turn","card":"AS","player":2}
                 {"type":"state_update","state":{...}}
Client sends:    {"type":"call","call":"Pass"}
                 {"type":"play","card":"QH"}
```

The room's WebSocket acts as a pub/sub channel. When any player acts, the server broadcasts the updated state to all connected clients. Bot turns stream in as individual messages (animatable) instead of one bulk response.

**Migration Phase 2 — Heartbeat + reconnect:**
- Server sends `{"type":"ping"}` every 10s
- Client responds with `{"type":"pong"}`
- On disconnect: client stores last known state, reconnects with `?room={id}&seat={index}&last_state_version=42`
- Server replays missed messages from an in-memory ring buffer (last 100 messages per room)

---


### Introduce Observability

Current state: `tracing::info!("...")` scattered through the code, no dashboard, no metrics. You only find out the server crashed when someone says "the game is down."

**Structured events (JSON, not strings):**

```rust
// Instead of:
tracing::info!("Bot action rejected at phase {:?}, player {}", phase, p);

// Do:
tracing::info!(
    bot_action_rejected,
    phase = ?table.phase,
    player = current,
    error = %e,
    room_id = %room_id,
);
```

**Three observability pillars:**

| Pillar | Tool | What we track |
|--------|------|---------------|
| **Logs** | `tracing` → file or stdout, shipped to Loki/Grafana | Room lifecycle, auth failures, bot errors, game start/end |
| **Metrics** | `metrics` crate + Prometheus endpoint at `/metrics` | Active rooms, active players, bot decision time p50/p95/p99, HTTP 4xx/5xx count |
| **Traces** | `tracing` spans with OpenTelemetry | Per-action trace: `handle_call → make_call → persist_state`, measure each step |

**Prometheus endpoint:**

```rust
use axum::routing::get;
use metrics_exporter_prometheus::PrometheusHandle;

let recorder = PrometheusBuilder::new().install_recorder()?;
let handle = recorder.handle();
app.route("/metrics", get(move || async move { handle.render() }));

// Increment counters:
metrics::counter!("rooms.active").increment(1);
metrics::histogram!("bot.decision_time_ms", duration.as_millis() as f64);
```

**Grafana dashboard panels:**
- Active rooms & players (gauge)
- Bot decision time (heatmap, p50/p95 lines)
- Error rate (4xx, 5xx per route)
- Room lifecycle (creations, starts, finishes per minute)

---

### Scaling 

**Current bottleneck:** `Arc<RwLock<HashMap<Uuid, GameRoom>>>` — all rooms share one lock. A slow bot turn (shouldn't happen, but) blocks all other rooms. Estimated limit of ~50 concurrent games. 
We introduce Redis in-memory cache to save room state. 

| Approach | What migrates | Trade-off |
|----------|---------------|-----------|
| **Redis hash per room** | `GAME:{room_id}` → full state JSON | Simple, but each bot turn = serialize + write + read + deserialize. Plus serialization cost on every turn. |
| **Local cache + Redis backup** | In-memory HashMap for active rooms, Redis for crash recovery | Best latency, but 2× memory. Only write to Redis after each complete trick (not every card). |
| **Redis pub/sub for cross-instance** | If we ever need >1 server | Each server instance subscribes to room channels. Players can connect to any instance. |

**Recommended: local cache + Redis backup.** Keep the current in-memory HashMap for speed. After each completed trick (every 4 cards), snapshot the full game state to a Redis hash. If a server restarts, rooms can be recovered from Redis.

**Redis data layout:**

```
GAME:{room_id} → Hash {
    "state": JSON-serialized GameRoom,
    "players": JSON array of {id, name, seat},
    "ttl": 3600  // 1 hour, rooms expire
}
ROOM_PLAYER:{room_id} → Set of player session IDs
```
 
 ### Miscellaneous
- Improved tutorial mode with hints and guides.
- Progress tracker and analytics: per-player stats, hand history, heatmaps of bidding/plays, and improvement suggestions.
- Improved UI: here we aim for a clean, simple aesthetic, focusing on functionality first. Once the features have been finalised, we are aiming for a more cosy, cute style similar to games like [Stardew Valley](https://www.stardewvalley.net/).
- Enhanced bots: predicting hands based on the bidding phase. 

## Tech Stack

- Backend: Rust
- Frontend: Svelte + Vite (TypeScript)
- UI: shadcn-svelte
- Database: Turso (SQLite)
- Build & tooling: Cargo for Rust backend, npm for Svelte frontend
- Deployment: Vercel (Frontend),

## Testing

The project maintains **57 automated tests** across three test suites — all passing with zero failures. Tests are split into unit tests (game-logic), bot AI tests, and integration tests (HTTP API).

### Running tests

```bash
cd backend
cargo test --workspace          # Run all tests (57 total)
cargo test -p game-core         # Game-core unit tests (37)
cargo test -p bridge-server     # Integration + bot tests (20)
cargo test -- --list            # List every test by name
```

### 1. Game-core unit tests (37 tests)

Located in `backend/crates/game-core/src/{bid.rs,game.rs,scoring.rs}`. No database or network needed — pure in-memory state machine assertions.

| Module | Tests | What it covers |
|--------|-------|----------------|
| `bid`        | 17 | Auction lifecycle: start → must outrank → 3 passes ends it. Bid ordering, display formatting, valid/invalid parsing, call abbreviations. Forces 1♣ when all pass. |
| `game`       | 14 | Deal gives 13 cards each, next deal rotates dealer, bidding phase transitions, partner selection (cannot pick own card), team 1/2 win detection at target sets. |
| `scoring`    | 7  | Higher rank beats lower, led-suit must be followed, trump beats non-trump, vulnerability state tracking. |

**Example — raised bets must outrank current highest bid** (`bid.rs`):

```rust
#[test]
fn test_auction_must_outrank() {
    let mut a = Auction::new();
    a.add_call(Call::Bid(Bid { level: 1, strain: Strain::Clubs }));
    assert!(a.add_call(Call::Bid(Bid { level: 1, strain: Strain::Clubs })).is_err());
    assert!(a.add_call(Call::Bid(Bid { level: 1, strain: Strain::Diamonds })).is_ok());
}
```

### 2. Bot AI tests (8 tests)

Located in `backend/crates/server/src/bot.rs`. Each test validates that the bot makes legal, reasonable decisions.

| Test | What it checks |
|------|----------------|
| `test_card_beats_led_suit` | Card of led suit beats off-suit |
| `test_card_beats_same_suit` | Higher rank beats lower of same suit |
| `test_card_beats_trump` | Trump card beats non-trump |
| `test_suit_scoring` | Scoring logic for card evaluation |
| `test_legal_plays_follow_suit` | Bot must follow suit when possible |
| `test_legal_plays_no_follow` | Bot can play any card when void |
| `test_legal_plays_cannot_lead_trump` | Bot cannot lead trump before broken |
| `test_decide_bid` | Bot produces a valid bid |

**Example — bot must follow suit** (`bot.rs`):

```rust
#[test]
fn test_legal_plays_follow_suit() {
    let hand = vec![
        Card { suit: Suit::Hearts, value: 2 },
        Card { suit: Suit::Hearts, value: 5 },
        Card { suit: Suit::Clubs, value: 10 },
    ];
    let led_suit = Suit::Hearts;
    let plays = legal_plays(&hand, led_suit, false);
    assert_eq!(plays.len(), 2);          // Only hearts
    assert!(plays.iter().all(|c| c.suit == Suit::Hearts));
}
```

### 3. HTTP Integration tests (12 tests)

Located in `backend/crates/server/tests/api_test.rs`. Each test starts a real Axum server with an in-memory SQLite database (no Turso dependency), runs full HTTP requests against it, and asserts on JSON responses.

| Test | What it verifies |
|------|------------------|
| `test_signup_and_login` | Full auth flow: signup → login → session token returned |
| `test_login_wrong_password` | Wrong password returns 401 |
| `test_signup_duplicate` | Duplicate username returns error |
| `test_signup_validates_input` | Empty/weak inputs rejected |
| `test_logout_invalidates_session` | Token cannot be reused after logout |
| `test_matches_empty` | GET /api/matches returns empty list for new user |
| `test_matches_requires_auth` | Unauthenticated match query returns 401 |
| `test_save_and_retrieve_match` | POST match → GET returns it with correct data |
| `test_save_match_dedup_by_room` | Same room_id cannot create duplicate match |
| `test_leaderboard_with_user` | Leaderboard includes logged-in user |
| `test_leaderboard_without_bots` | Bot IDs 1,2,3 excluded from leaderboard |
| `test_health_check` | Server responds to basic requests |

**Example — full auth + match lifecycle** (`api_test.rs`):

```rust
#[tokio::test]
async fn test_signup_and_login() {
    let (app_state, _tmp) = new_temp_db().await;
    let app = create_app(app_state);

    // Signup
    let res = request(&app, "POST", "/api/auth/signup",
        r#"{"username":"alice","password":"secret123"}"#).await;
    assert_eq!(res.status(), 200);

    // Login
    let res = request(&app, "POST", "/api/auth/login",
        r#"{"username":"alice","password":"secret123"}"#).await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = serde_json::from_slice(&res.body()).unwrap();
    assert!(body["ok"].as_bool().unwrap());
    assert!(body["token"].as_str().unwrap().len() > 10);
}
```

### Test Infrastructure

- **No Turso dependency** — integration tests use `Database::open(":memory:")` with tempfile SQLite via the `new_temp()` helper in `db.rs`.
- **Each test is isolated** — `#[tokio::test]` runs with its own app instance and fresh database.
- **Bot tests use mock state** — bot AI is tested in isolation without needing a full game loop or network.
- **Quick feedback** — the full suite runs in under **1 second** on a local machine.

## Architecture

```
[Browser] ── HTTP REST ──► [Rust/Axum on Hetzner CX23] ──► [Turso/SQLite]
               polling                     │
              2s intervals                HashMap<Uuid, GameRoom>
                                          in-memory, Arc<RwLock>
```

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
| 5    | Bot tuning: improve difficulty parameters and add small randomized tie-breakers |
| 6    | Prototype WebSocket room endpoint and basic client connect/reconnect |
| 7    | Replace polling with WebSocket-driven state updates in the game UI |
| 8    | Implement matchmaking and lobby improvements (join/create/seat flow) |
| 9    | Progress tracker backend: match saving, stats schema, and APIs |
| 10   | Frontend analytics: match history viewer, basic player stats UI |
| 11   | Observability: add structured tracing, Prometheus metrics, `/metrics` endpoint |
| 12   | Scaling & recovery: Redis snapshot/backups for rooms, staging load testing |


Detailed day-by-day tasks can be found in our project log. 

---
