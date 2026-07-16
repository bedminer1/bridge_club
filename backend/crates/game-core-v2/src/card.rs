

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Card(u8);

impl Card {
	pub const fn new(suit: u8, rank: u8) -> Self {
		Card((rank << 2) | (suit & 0b11))
	}
	pub const fn suit(self) -> u8 { self.0 & 0b11 }
	pub const fn rank(self) -> u8 { (self.0 >> 2) & 0b111 }
	pub const fn index(self) -> u8 { self.0.suit * 13 + self.0.rank }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Hand(u64);

impl Hand {
	// Checks if hand has a card
	pub fn has(&self, card: Card) -> bool {
		(self.0 & (1u64 << card.index()))
	}

	// PLay card from a hand
	pub fn remove(&mut self, card: Card) {
		self.0 &= !(1u64 << card.index());
	}
}