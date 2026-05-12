#include "Card.hpp"
#include <string>

std::string Card::toString() const {
	std::string rankStr;
	switch (rank) {
		case Rank::Jack:  rankStr = "J"; break;
	    case Rank::Queen: rankStr = "Q"; break;
	    case Rank::King:  rankStr = "K"; break;
	    case Rank::Ace:   rankStr = "A"; break;
	    default:          rankStr = std::to_string(static_cast<int>(rank)); break;
	}
	
	std::string suitStr;
	switch (suit) {
        case Suit::Clubs:    suitStr = "♣"; break;
        case Suit::Diamonds: suitStr = "♦"; break;
        case Suit::Hearts:   suitStr = "♥"; break;
        case Suit::Spades:   suitStr = "♠"; break;
    }

    return rankStr + suitStr;
}
