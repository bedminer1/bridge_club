//! Headless throughput benchmark for the bridge engine.
//!
//! Runs 4 bots against each other in a tight loop, measuring
//! games per second and decisions per second. No I/O, no network,
//! no allocations beyond the game state itself.
//!
//! Usage:
//!   cargo run --release --bin bench          # default: 3-second run
//!   cargo run --release --bin bench -- 10     # 10-second run
//!   cargo run --release --bin bench -- 30 medium  # 30s, Medium bots

use std::time::Instant;

use game_core::{GamePhase, Table};
use bridge_server::bot::{auto_decide, BotAction, BotDifficulty};

fn apply_bot_action(table: &mut Table, action: BotAction) -> Result<(), &'static str> {
    match action {
        BotAction::Call(call) => table.make_call(call),
        BotAction::SelectPartner(card) => table.select_partner(card),
        BotAction::PlayCard(card) => table.play_card(card),
    }
}

/// Run a single game with 4 bots. Returns the number of decisions made
/// (calls + card plays + partner selections).
fn run_one_game(difficulty: BotDifficulty) -> u64 {
    let mut table = Table::new(["Bot0", "Bot1", "Bot2", "Bot3"]);
    table.deal();

    let mut decisions: u64 = 0;

    loop {
        match table.phase {
            GamePhase::Finished => break,
            GamePhase::Dealing | GamePhase::Scoring => break,
            _ => {
                let action = auto_decide(&table, difficulty)
                    .unwrap_or_else(|_| panic!("Bot failed at phase {:?}", table.phase));
                apply_bot_action(&mut table, action)
                    .unwrap_or_else(|e| panic!("Bot action rejected: {}", e));
                decisions += 1;
            }
        }
    }

    decisions
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let duration_secs: f64 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3.0);

    let difficulty = match args.get(2).map(|s| s.as_str()) {
        Some("medium") | Some("Medium") => BotDifficulty::Medium,
        _ => BotDifficulty::Easy,
    };

    println!("=== Bridge Engine Benchmark ===");
    println!(
        "Difficulty: {:?}  |  Duration: {}s",
        difficulty, duration_secs
    );
    println!();

    // Warmup: 500 games (JIT warmup, cache priming)
    print!("Warming up (500 games)... ");
    let warmup_start = Instant::now();
    for _ in 0..500 {
        run_one_game(difficulty);
    }
    let warmup_elapsed = warmup_start.elapsed();
    println!(
        "done in {:.2}s ({:.0} games/s)",
        warmup_elapsed.as_secs_f64(),
        500.0 / warmup_elapsed.as_secs_f64()
    );

    // Timed run
    println!();
    println!("Running for {}s...", duration_secs);
    let start = Instant::now();
    let deadline = start + std::time::Duration::from_secs_f64(duration_secs);

    let mut games: u64 = 0;
    let mut decisions: u64 = 0;

    // Run in batches of 1000 to avoid Instant::now() overhead
    loop {
        let batch_start = Instant::now();
        if batch_start >= deadline {
            break;
        }

        for _ in 0..1000 {
            if Instant::now() >= deadline {
                break;
            }
            decisions += run_one_game(difficulty);
            games += 1;
        }
    }

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();

    println!();
    println!("=== Results ===");
    println!("  Games:      {}", games);
    println!("  Decisions:  {}", decisions);
    println!("  Wall time:  {:.3}s", secs);
    println!();
    println!("  Games/sec:      {:.0}", games as f64 / secs);
    println!("  Decisions/sec:  {:.0}", decisions as f64 / secs);
    println!(
        "  Avg μs/game:    {:.1}",
        (secs * 1_000_000.0) / games as f64
    );
    println!(
        "  Avg decisions/game: {:.1}",
        decisions as f64 / games as f64
    );
}
