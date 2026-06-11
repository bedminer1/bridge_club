/** Shared player display utilities. */

export const playerIDToColor = new Map<number, string>([
    [1, "red"],
    [2, "blue"],
    [3, "green"],
    [4, "yellow"],
]);

export function playerName(game: any, playerId: number): string {
    return game?.Players?.find((p: any) => p.ID === playerId)?.Username ?? `P${playerId}`;
}
