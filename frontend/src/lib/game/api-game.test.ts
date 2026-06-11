import { describe, it, expect } from 'vitest';
import { frontendCardToApiCard, apiStateToGame } from './api-game';

describe('frontendCardToApiCard', () => {
  it('converts a frontend card to API format', () => {
    const card = { Suit: 'Club', Value: 7, Rank: '7' };
    expect(frontendCardToApiCard(card)).toEqual({ suit: 'Clubs', rank: 'Seven' });
  });

  it('handles face cards', () => {
    const card = { Suit: 'Spades', Value: 11, Rank: 'J' };
    expect(frontendCardToApiCard(card)).toEqual({ suit: 'Spades', rank: 'Jack' });
  });
});

describe('apiStateToGame', () => {
  const dealingState = {
    phase: 'Dealing',
    hands: ['SA SK SQ', 'HA HK HQ', 'DA DK DQ', 'CA CK CQ'],
    currentPlayer: 0,
    betSize: 0,
    trumpSuit: '',
    betWinner: null,
    partnerIdx: null,
    setsWon: [0, 0, 0, 0],
    completedSetCount: 0,
    isFinished: false,
    currentTrickCards: [],
    currentTrickStartPlayer: 0,
    previousTrickCards: [],
    previousTrickWinner: null,
    previousTrickStartPlayer: 0,
    callHistory: [],
    callHistoryStartPlayer: 0,
    partnerCard: null,
    trumpPlayed: false,
    leadSuit: null,
    completedSets: [],
    playerNames: ['Alice', 'Bob', 'Bot-Alpha', 'Bot-Beta'],
  };

  it('creates 4 players from state', () => {
    const game = apiStateToGame(dealingState, 'room-1');
    expect(game.Players).toHaveLength(4);
    expect(game.Players[0].Username).toBe('Alice');
    expect(game.Players[1].Username).toBe('Bob');
  });

  it('assigns IsBot based on name prefix', () => {
    const game = apiStateToGame(dealingState, 'room-1');
    expect(game.Players[0].IsBot).toBe(false);  // Alice
    expect(game.Players[2].IsBot).toBe(true);   // Bot-Alpha
  });

  it('sets IsBettingPhase based on phase string', () => {
    const biddingState = { ...dealingState, phase: 'Bidding' };
    const game = apiStateToGame(biddingState, 'room-1');
    expect(game.IsBettingPhase).toBe(true);
    expect(game.IsPartnerSelectionPhase).toBe(false);
  });

  it('detects partner selection phase', () => {
    const partnerState = { ...dealingState, phase: 'PartnerSelection' };
    const game = apiStateToGame(partnerState, 'room-1');
    expect(game.IsPartnerSelectionPhase).toBe(true);
    expect(game.IsBettingPhase).toBe(false);
  });
});
