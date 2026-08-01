#[cfg(test)]
mod tests {
    use crate::card::*;

    #[test]
    fn test_card_roundtrip() {
        let c = Card::new(Suit::Spades, Rank::Ace);
        let json = serde_json::to_string(&c).unwrap();
        let c2: Card = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn test_card_deser_both_forms() {
        for (suit, rank, j_suit, j_rank) in [
            (Suit::Spades, Rank::Ace, "Spades", "Ace"),
            (Suit::Hearts, Rank::Ten, "Hearts", "Ten"),
            (Suit::Diamonds, Rank::Five, "Diamonds", "5"),
            (Suit::Clubs, Rank::Two, "Club", "2"),
        ] {
            let json = format!(r#"{{"Suit":"{}","Value":{},"Rank":"{}"}}"#, j_suit, rank as u8, j_rank);
            let card: Card = serde_json::from_str(&json).unwrap();
            assert_eq!(card.suit(), suit);
            assert_eq!(card.rank(), rank);
        }
    }

    #[test]
    fn test_card_not_zero() {
        let card: Card = serde_json::from_str(r#"{"Suit":"Spades","Value":14,"Rank":"Ace"}"#).unwrap();
        assert_eq!(card, Card::new(Suit::Spades, Rank::Ace));
    }
}
