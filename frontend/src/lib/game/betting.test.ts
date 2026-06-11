import { describe, it, expect } from 'vitest';
import { isLegalRaise } from './betting';

describe('isLegalRaise', () => {
  const baseGame = {
    BetSize: 1,
    Trump: 'Club',
    Players: [] as any[],
    Moves: [] as any[],
    IsBettingPhase: true,
  } as any;

  it('allows raising with higher bet size', () => {
    expect(isLegalRaise(baseGame, 2, 'Club')).toBe(true);
  });

  it('allows raising to same bet with higher suit priority', () => {
    expect(isLegalRaise(baseGame, 1, 'Spades')).toBe(true);
    expect(isLegalRaise(baseGame, 1, 'Heart')).toBe(true);
  });

  it('rejects same bet + lower suit priority', () => {
    expect(isLegalRaise(baseGame, 1, 'Club')).toBe(false);
  });

  it('rejects lower bet size regardless of suit', () => {
    expect(isLegalRaise(baseGame, 0, 'Spades')).toBe(false);
  });

  it('allows raise from NoTrump (Club=0) to anything', () => {
    const noTrumpGame = { ...baseGame, BetSize: 0, Trump: 'Club' };
    expect(isLegalRaise(noTrumpGame, 1, 'Club')).toBe(true);
  });

  it('handles Spades > Hearts > Diamond > Club priority', () => {
    const highGame = { ...baseGame, BetSize: 2, Trump: 'Diamond' };
    expect(isLegalRaise(highGame, 2, 'Heart')).toBe(true);
    expect(isLegalRaise(highGame, 2, 'Spades')).toBe(true);
    expect(isLegalRaise(highGame, 2, 'Diamond')).toBe(false);
    expect(isLegalRaise(highGame, 2, 'Club')).toBe(false);
  });
});
