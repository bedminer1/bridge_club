#include "Deck.hpp"
#include "Player.hpp"
#include <iostream>
#include <string>

int main() {
	Deck deck;
	deck.shuffle();
	
	std::vector<Player> players = {
		Player("A"),
		Player("B"),
		Player("C"),
		Player("D"),
	};
	
	
	for (Player& p : players) {
        for (int i = 0; i < 13; ++i) {
            p.receiveCard(deck.draw());
        }
        p.sortHand();
    }
    
    for (const auto& p : players) {
    	std::cout << p.getHandString() << std::endl;
    }
    
	return 0;
}