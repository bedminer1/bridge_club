/** Shared player display utilities. */

export const playerIDToColor = new Map<number, string>([
    [1, "[var(--red)]"],
    [2, "[var(--blue)]"],
    [3, "[var(--yellow)]"],
    [4, "[var(--green)]"],
]);

export function playerName(game: any, playerId: number): string {
    return game.Players?.find((p: any) => p.ID === playerId)?.Username ?? `P${playerId}`;
}
