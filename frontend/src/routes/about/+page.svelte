<script lang="ts">
    import { Gamepad2, Crown, Github } from "@lucide/svelte"
    import { suitToSymbol } from "$lib/utils"

    const suits = { c: "Club", d: "Diamond", h: "Heart", s: "Spades" }
</script>

<div class="flex flex-col items-center w-full pt-20 px-4">
    <h1 class="text-2xl font-bold mb-2">Bridge Club</h1>
    <p class="text-sm text-muted-foreground mb-8">Singapore Bridge</p>

    <div class="w-full max-w-2xl flex flex-col gap-4 text-sm">
        <!-- How to Play -->
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="font-semibold mb-3 text-lg">How to Play</h2>

            <!-- Sets -->
            <div class="mb-6">
                <h3 class="text-accent text-base font-medium mb-1">{suitToSymbol.get('Spades')} Sets</h3>
                <p class="primary leading-relaxed">
                    4 players each play one card. The <strong class="text-foreground">highest card of the led suit</strong> wins the set.
                    The winner leads the next set. Play up to 13 sets per round.
                </p>
                <div class="flex gap-2 items-center mt-2 text-s">
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card">K{suitToSymbol.get('Spades')} led</span>
                    <span class="text-muted-foreground">&rarr;</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">2{suitToSymbol.get('Heart')}</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">3{suitToSymbol.get('Heart')}</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">4{suitToSymbol.get('Heart')}</span>
                    <span class="text-green text-s ml-1">
                        K♠ won
                    </span>
                </div>
            </div>

            <!-- Trump -->
            <div class="mb-6">
                <h3 class="text-accent text-base font-medium mb-1">{suitToSymbol.get('Heart')} Trump Suit</h3>
                <p class="primary text-s leading-relaxed">
                    A <strong class="text-foreground">trump suit</strong> is chosen before play begins.
                    Trump <strong class="text-foreground">beats any card of any other suit</strong>.
                    If you can't follow the led suit, you may play a trump to steal the set.
                    Trump cannot be <strong class="text-foreground">led</strong> until it has been played on another suit first (trump <em class="text-foreground">broken</em>).
                    Exception: if you're <strong class="text-foreground">void</strong> in the led suit, you may lead trump even before it's broken.
                </p>
                <div class="flex gap-2 items-center mt-2 text-s">
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card">A{suitToSymbol.get('Club')} led (♣)</span>
                    <span class="text-muted-foreground">&rarr;</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">K{suitToSymbol.get('Club')}</span>
                    <span class="rounded border border-border px-1.5 py-0.5 bg-card opacity-60">Q{suitToSymbol.get('Club')}</span>
                    <span class="rounded border border-b-2 border-red-500 px-1.5 py-0.5 bg-card font-bold">2{suitToSymbol.get('Heart')} <span class="text-red-500">trump!</span></span>
                    <span class="text-green text-s ml-1">
                        2♥ won
                    </span>
                </div>
            </div>

            <!-- Bidding -->
            <div class="mb-6">
                <h3 class="text-accent text-base font-medium mb-1">{suitToSymbol.get('Diamond')} Bidding</h3>
                <p class="primary text-s leading-relaxed">
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
            <div class="mb-6">
                <h3 class="text-accent text-base font-medium mb-1">{suitToSymbol.get('Club')} Partners</h3>
                <p class="primary text-s leading-relaxed">
                    The bet winner selects a <strong class="text-foreground">partner card</strong>
                    from the 39 cards they don't hold. The player who holds that card is their partner.
                    The <strong class="text-foreground">two partners form Team 1</strong>;
                    the other two players are Team 2. 
                    The bet winner and the other 2 players that are not the partner will be in the dark about who is the partner until the card is played, but detective work can be done to deduce it.
                </p>
            </div>

            <!-- Scoring -->
            <div>
                <h3 class="text-accent text-base font-medium mb-1">{suitToSymbol.get('Spades')} Scoring</h3>
                <p class="primary text-s leading-relaxed">
                	Both teams have a target number of sets to win:
                </p>
                <ul class="text-s text-primary mt-1 space-y-0.5 ml-3 list-disc [&>li]:pl-1">
                    <li><strong class="text-foreground">Team 1</strong> wins if they take <strong class="tabular-nums">6 + bid_size</strong> or more sets.</li>
                    <li><strong class="text-foreground">Team 2</strong> wins if they take <strong class="tabular-nums">8 &minus; bid_size</strong> or more sets.</li>
                    <li>If <strong class="text-accent">Tutorial Mode</strong> is enabled, <strong class="text-foreground">Elo</strong> will not be affected.</li>
                </ul>
            </div>
        </div>

        <!-- About -->
        <div class="rounded-lg border border-border bg-card p-4">
            <h2 class="text-lg font-semibold mb-2">About</h2>
            <p class="primary">
                Built as a learning project. Open source on
                <a href="https://github.com/bedminer1/bridge_club" target="_blank" class="text-accent hover:underline">
                    GitHub <Github class="inline w-3 h-3" />
                </a>.
            </p>
        </div>

        <!-- Terms of Use -->
        <div id="terms" class="rounded-lg border border-border bg-card p-4">
            <h2 class="text-lg font-semibold mb-2">Terms of Use</h2>
            <p class="text-sm text-muted-foreground leading-relaxed">
                Bridge Club is a recreational card game. By using this site you agree to:
            </p>
            <ul class="text-sm text-muted-foreground mt-2 space-y-1 ml-4 list-disc">
                <li>Play fairly — no cheating, botting, or exploiting bugs.</li>
                <li>No abusive or harassing behavior toward other players.</li>
                <li>We reserve the right to suspend or ban accounts that violate these terms.</li>
                <li>This service is provided as-is with no guarantees of uptime or data retention.</li>
            </ul>
        </div>

        <!-- Privacy Policy -->
        <div id="privacy" class="rounded-lg border border-border bg-card p-4">
            <h2 class="text-lg font-semibold mb-2">Privacy Policy</h2>
            <p class="text-sm text-muted-foreground leading-relaxed">
                We collect only what's needed to run the game:
            </p>
            <ul class="text-sm text-muted-foreground mt-2 space-y-1 ml-4 list-disc">
                <li><strong>Email address</strong> — from Google sign-in, used only for account identification.</li>
                <li><strong>Username</strong> — your chosen display name.</li>
                <li><strong>Password</strong> — if using email sign-up, stored as a SHA-256 hash, never in plaintext. Transmitted over HTTPS only.</li>
                <li><strong>Game history</strong> — match results, cards played, Elo rating.</li>
            </ul>
            <p class="text-sm text-muted-foreground mt-3">
                No data is sold, shared with third parties, or used for marketing.
                You can request account deletion by contacting the developer.
            </p>
        </div>

        <!-- Links -->
        <div class="flex gap-4 justify-center mt-4 mb-10">
            <a href="/" class="flex items-center gap-2 text-sm text-accent hover:underline">
                <Gamepad2 class="w-4 h-4" /> Play
            </a>
            <a href="/leaderboard" class="flex items-center gap-2 text-sm text-accent hover:underline">
                <Crown class="w-4 h-4" /> Leaderboard
            </a>
        </div>
    </div>
</div>
