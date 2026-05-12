#pragma once

#include <string>

enum class Suit {
	Clubs,
	Diamonds,
	Hearts,
	Spades
};

enum class Rank {
	Two = 2, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace
};

struct Card {
	Suit suit;
	Rank rank;
	
	Card(Suit s, Rank r) : suit(s), rank(r) {}
	std::string toString() const;
};