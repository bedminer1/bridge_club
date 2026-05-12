#include "Bid.hpp"
#include <cctype>

std::optional<Bid> Bid::parse(std::string_view input) {
	if (input.size() != 2) {
		return std::nullopt;
	}
	
	int level = input[0] - '0';
	Suit suit;
	switch (input[1]) {
		case 'C':
			suit = Suit::Clubs;
			break;
		case 'D':
			suit = Suit::Diamonds;
			break;
		case 'H':
			suit = Suit::Hearts;
			break;
		case 'S':
			suit = Suit::Spades;
			break;
		default:
			return std::nullopt;
	}
	
	return Bid(level, suit);
}