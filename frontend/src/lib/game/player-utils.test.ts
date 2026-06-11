import { describe, it, expect } from 'vitest';
import { playerName, playerIDToColor } from './player-utils';

describe('playerIDToColor', () => {
  it('maps player 1 to red', () => {
    expect(playerIDToColor.get(1)).toBe('[var(--red)]');
  });
  it('maps player 2 to blue', () => {
    expect(playerIDToColor.get(2)).toBe('[var(--blue)]');
  });
  it('maps player 3 to yellow', () => {
    expect(playerIDToColor.get(3)).toBe('[var(--yellow)]');
  });
  it('maps player 4 to green', () => {
    expect(playerIDToColor.get(4)).toBe('[var(--green)]');
  });
});

describe('playerName', () => {
  const mockGame = {
    Players: [
      { ID: 1, Username: 'Alice' },
      { ID: 2, Username: 'Bob' },
      { ID: 3, Username: 'Bot-Alpha' },
      { ID: 4, Username: 'Bot-Beta' },
    ],
  };

  it('returns the username for a known player', () => {
    expect(playerName(mockGame, 1)).toBe('Alice');
    expect(playerName(mockGame, 2)).toBe('Bob');
  });

  it('returns P{id} for unknown player ID', () => {
    expect(playerName(mockGame, 99)).toBe('P99');
  });

  it('handles empty Players array', () => {
    expect(playerName({ Players: [] }, 1)).toBe('P1');
  });

  it('handles null game', () => {
    expect(playerName(null, 1)).toBe('P1');
  });
});
