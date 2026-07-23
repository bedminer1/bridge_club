const SUITS = ["CLUB", "DIAMOND", "HEART", "SPADE"] as const;
const RANK_SUFFIXES = [
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "10",
    "11-JACK",
    "12-QUEEN",
    "13-KING",
] as const;

const CARD_ASSET_PATHS = [
    ...SUITS.flatMap((suit) => RANK_SUFFIXES.map((rank) => `/cards/${suit}-${rank}.svg`)),
    "/cards/JOKER-1.svg",
    "/cards/JOKER-2.svg",
    "/cards/JOKER-3.svg",
];

let preloadStarted = false;

export function preloadCardAssets(): void {
    if (typeof window === "undefined" || preloadStarted) return;
    preloadStarted = true;

    for (const path of CARD_ASSET_PATHS) {
        const image = new Image();
        image.decoding = "async";
        image.src = path;
    }
}
