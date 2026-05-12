#pragma once

#include "Card.hpp"
#include <optional>
#include <string_view>

struct Bid {
	int level;
	Suit suit;
	
	Bid(int l, Suit s) : level(l), suit(s) {}
	
	bool operator>(const Bid& other) const {
		if (level != other.level) {
			return level > other.level;
		}
		return suit > other.suit;
	}
	
	static std::optional<Bid> parse(std::string_view input);
};