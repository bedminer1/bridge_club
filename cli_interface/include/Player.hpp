#pragma once

#include "Card.hpp"
#include <string>
#include <vector>

class Player {
private:
	std::string name;
	std::vector<Card> cards;
	
public:
	Player(std::string n);
	void receiveCard(Card card);
	int handSize();
	void sortHand();
	std::string getHandString() const; // think of const as read-only
};