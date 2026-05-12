#include <iostream>
#include <string>
#include "Deck.hpp"

int main() {
	Deck deck;
	
	std::cout << "Shuffling deck..." << std::endl;
	deck.shuffle();
	
	std::cout << "Dealt cards: " << std::endl;
    while (deck.size() > 0) {
        Card c = deck.draw();
        std::cout << c.toString() << " ";

        // Add a newline every 13 cards (one hand)
        if (deck.size() % 13 == 0) {
            std::cout << std::endl;
        }
    }
	return 0;
}