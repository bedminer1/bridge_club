#pragma once

#include "Card.hpp"
#include <vector>

class Deck {
private:
	std::vector<Card> cards;
	
public:
	// equivalent to new() -> Self
	Deck();
	
	// reorders vector
	void shuffle();
	
	// removes and returns last card
	Card draw();
	
	// 'const' -> deck's data unchanged
	int size() const;
};