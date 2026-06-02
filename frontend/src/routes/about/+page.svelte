<script lang="ts">
    import { Gamepad2, Crown, Github } from "@lucide/svelte"
    import { suitToSymbol } from "$lib/utils"

    const suits = { c: "Club", d: "Diamond", h: "Heart", s: "Spades" }
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-2">Bridge Club</h1>
    <p class="text-sm text-muted-foreground mb-8">Singapore Bridge</p>

    <div class="w-full max-w-lg flex flex-col gap-4 text-sm">
        <!-- How to Play -->
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-3">How to Play</h2>

            <!-- Sets -->
            <div class="mb-3">
                <h3 class="text-accent font-medium mb-1">{suitToSymbol.get('Spades')} Sets</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                    4 players each play one card. The <strong class="text-foreground">highest card of the led suit</strong> wins the set.
                    The winner leads the next set. Play up to 13 sets per round.
                </p>
                <div class="flex gap-2 items-center mt-2 text-[11px]">
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card">K{suitToSymbol.get('Spades')} led</span>
                    <span class="text-muted-foreground">&rarr;</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">2{suitToSymbol.get('Heart')}</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">3{suitToSymbol.get('Heart')}</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">4{suitToSymbol.get('Heart')}</span>
                    <span class="text-green text-xs ml-1">
                        K♠ won
                    </span>
                </div>
            </div>

            <!-- Trump -->
            <div class="mb-3">
                <h3 class="text-accent font-medium mb-1">{suitToSymbol.get('Heart')} Trump Suit</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                    A <strong class="text-foreground">trump suit</strong> is chosen before play begins.
                    Trump <strong class="text-foreground">beats any card of any other suit</strong>.
                    If you can't follow the led suit, you may play a trump to steal the set.
                    Trump cannot be <strong class="text-foreground">led</strong> until it has been played on another suit first (<em class="text-muted-foreground">trump broken</em>).
                    Exception: if you're <strong class="text-foreground">void</strong> in the led suit, you may lead trump even before it's broken.
                </p>
                <div class="flex gap-2 items-center mt-2 text-[11px]">
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card">A{suitToSymbol.get('Club')} led (♣)</span>
                    <span class="text-muted-foreground">&rarr;</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">K{suitToSymbol.get('Club')}</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">Q{suitToSymbol.get('Club')}</span>
                    <span class="rounded border border-b-2 border-red-500 px-1.5 py-0.5 bg-card font-bold">2{suitToSymbol.get('Heart')} <span class="text-red-500">trump!</span></span>
                    <span class="text-green text-xs ml-1">
                        2♥ won
                    </span>
                </div>
            </div>

            <!-- Bidding -->
            <div class="mb-3">
                <h3 class="text-accent font-medium mb-1">{suitToSymbol.get('Diamond')} Bidding</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                    Each player has the chance to bid for the trump suit.
                    A bid pairs a <strong class="text-foreground">set target</strong> (how many sets your team will take) with a
                    <strong class="text-foreground">trump suit</strong>.
                    To raise, you must bid higher sets <em>or</em> the same sets with a higher suit
                    (<span class="tabular-nums">♣ &lt; ♦ &lt; ♥ &lt; ♠</span>).
                    Pass if you don't want to bid. The <strong class="text-foreground">highest bid wins the betting phase</strong>
                    — their chosen suit becomes trump for the round.
                </p>
            </div>

            <!-- Partners -->
            <div class="mb-3">
                <h3 class="text-accent font-medium mb-1">{suitToSymbol.get('Club')} Partners</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                    The bet winner selects a <strong class="text-foreground">partner card</strong>
                    from the 39 cards they don't hold. The player who holds that card is their partner.
                    The <strong class="text-foreground">two partners form Team 1</strong>;
                    the other two players are Team 2. 
                    The bet winner and the other 2 players that are not the partner will be in the dark about who is the partner until the card is played, but detective work can be done to deduce it.
                </p>
            </div>

            <!-- Scoring -->
            <div>
                <h3 class="text-accent font-medium mb-1">{suitToSymbol.get('Spades')} Scoring</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                	Both teams have a target number of sets to win:
                </p>
                <ul class="text-xs text-muted-foreground mt-1 space-y-0.5 ml-3 list-disc [&>li]:pl-1">
                    <li><strong class="text-foreground">Team 1</strong> wins if they take <strong class="tabular-nums">6 + bid_size</strong> or more sets.</li>
                    <li><strong class="text-foreground">Team 2</strong> wins if they take <strong class="tabular-nums">8 &minus; bid_size</strong> or more sets.</li>
                    <li><strong class="text-foreground">Elo</strong> changes only in <strong class="text-accent">Hidden Mode Only</strong> games. Open games are practice &mdash; no rating change.</li>
                </ul>
            </div>
        </div>

        <!-- About -->
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-2">About</h2>
            <p class="text-muted-foreground">
                Built as a learning project. Open source on
                <a href="https://github.com/bedminer1/bridge_club" target="_blank" class="text-accent hover:underline">
                    GitHub <Github class="inline w-3 h-3" />
                </a>.
            </p>
        </div>

        <!-- Links -->
        <div class="flex gap-4 justify-center mt-4">
            <a href="/" class="flex items-center gap-2 text-sm text-accent hover:underline">
                <Gamepad2 class="w-4 h-4" /> Play
            </a>
            <a href="/leaderboard" class="flex items-center gap-2 text-sm text-accent hover:underline">
                <Crown class="w-4 h-4" /> Leaderboard
            </a>
        </div>
    </div>
</div>
