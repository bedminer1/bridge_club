#include "Player.hpp"
#include <algorithm>
#include <sstream>

Player::Player(std::string n) : name(n) {}

void Player::receiveCard(Card card) {
	cards.push_back(card);
}

int Player::handSize() {
	return static_cast<int>(cards.size());
}

// Suit then Rank
void Player::sortHand() {
	std::sort(cards.begin(), cards.end(), [](const Card& a, const Card& b) {
		if (a.suit != b.suit) {
			return a.suit < b.suit;
		}
		return a.rank < b.rank;
	});
}

std::string Player::getHandString() const {
	std::ostringstream builder;
	builder << name << ": ";
	
	for (const auto& card : cards) {
		builder << card.toString() << " ";
	}
	
	return builder.str();
}