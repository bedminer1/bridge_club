//! Correctness check: run N games and verify they all complete properly.
use game_core::{GamePhase, Table};
use bridge_server::bot::{auto_decide, BotAction, BotDifficulty};

fn apply_bot_action(table: &mut Table, action: BotAction) -> Result<(), &'static str> {
    match action {
        BotAction::Call(call) => table.make_call(call),
        BotAction::SelectPartner(card) => table.select_partner(card),
        BotAction::PlayCard(card) => table.play_card(card),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10_000);
    let difficulty = BotDifficulty::Medium;

    let mut team1_wins = 0u64;
    let mut errors = 0u64;

    for i in 0..n {
        let mut table = Table::new(["B0", "B1", "B2", "B3"]);
        table.deal();
        loop {
            match table.phase {
                GamePhase::Finished => {
                    // Determine winner
                    let bw = table.bet_winner.unwrap();
                    let partner = table.partner_idx;
                    let t1 = table.sets_won[bw]
                        + partner.map(|p| table.sets_won[p]).unwrap_or(0);
                    let t2: u8 = (0..4)
                        .filter(|&j| j != bw && partner.map_or(true, |p| j != p))
                        .map(|j| table.sets_won[j])
                        .sum();
                    if t1 >= 6 + table.bet_size {
                        team1_wins += 1;
                    } else if t2 < 8 - table.bet_size {
                        // Shouldn't happen if check_win_condition is correct
                    }
                    break;
                }
                GamePhase::Dealing | GamePhase::Scoring => break,
                _ => {
                    match auto_decide(&table, difficulty) {
                        Ok(action) => {
                            if let Err(e) = apply_bot_action(&mut table, action) {
                                eprintln!("Game {i}: {e}");
                                errors += 1;
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Game {i}: bot error: {e}");
                            errors += 1;
                            break;
                        }
                    }
                }
            }
        }

        if i > 0 && i % 10_000 == 0 {
            eprintln!("  verified {i} games, {errors} errors so far");
        }
    }

    println!("Verified {n} games:");
    println!("  Errors:       {errors}");
    println!("  Team1 (better) wins: {team1_wins} ({:.1}%)",
        team1_wins as f64 / n as f64 * 100.0);
    println!("  Team2 wins:   {} ({:.1}%)",
        n - team1_wins - errors,
        (n - team1_wins - errors) as f64 / n as f64 * 100.0);
}
