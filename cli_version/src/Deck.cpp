#include "Deck.hpp"
#include <algorithm> // for std::shuffle
#include <random> 

Deck::Deck() {
	// pre-allocate memory
	cards.reserve(52);
	
	for (Suit s: {Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades}) {
		for (Rank r: {Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King, Rank::Ace}) {
            cards.push_back(Card(s, r));          
        }
	}
}

void Deck::shuffle() {
	// get a seed
	std::random_device rd;
	
	// initialize random number generator
	std::mt19937 g(rd());
	
	std::shuffle(cards.begin(), cards.end(), g);
}

Card Deck::draw() {
	Card dealt_card = cards.back();
	cards.pop_back();
	return dealt_card;
}

int Deck::size() const {
	// cast to int from unsigned long
	return static_cast<int>(cards.size());
}