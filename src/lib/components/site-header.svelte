<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";

    import { toggleMode } from "mode-watcher";
    import { Gamepad2, Crown, Info, Settings, Bell, User } from "@lucide/svelte";
    import { page } from "$app/state";

    import { headerState } from "$lib/game/header-state.svelte";
</script>

<header
    class="fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-6 py-3 border-b border-border bg-background/80 backdrop-blur-sm"
>
    <!-- Left section -->
    <div class="flex items-center gap-2">
        <!-- Brand + separator -->
        <a href="/" class="font-bold text-base text-foreground hover:text-accent transition-colors no-underline mr-3">
            Bridge Club
        </a>
        <div class="w-px h-5 bg-border mr-3"></div>

        <!-- Nav icons -->
        <div class="flex items-center gap-3">
            <a
                href="/"
                class="p-1.5 rounded-md {page.url.pathname === '/' ? 'text-accent bg-accent/10' : 'text-muted-foreground'} hover:text-accent hover:bg-accent/10 transition-colors"
                title="Play"
            >
                <Gamepad2 class="w-4 h-4" />
            </a>
            <a
                href="/leaderboard"
                class="p-1.5 rounded-md {page.url.pathname === '/leaderboard' ? 'text-accent bg-accent/10' : 'text-muted-foreground'} hover:text-accent hover:bg-accent/10 transition-colors"
                title="Leaderboard"
            >
                <Crown class="w-4 h-4" />
            </a>
            <a
                href="/about"
                class="p-1.5 rounded-md {page.url.pathname === '/about' ? 'text-accent bg-accent/10' : 'text-muted-foreground'} hover:text-accent hover:bg-accent/10 transition-colors"
                title="About"
            >
                <Info class="w-4 h-4" />
            </a>
        </div>

        <!-- Settings popover (only when game is active) -->
        {#if headerState.game}
            <Popover.Root>
                <Popover.Trigger>
                    <button class="p-1.5 rounded-md text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors ml-1">
                        <Settings class="w-4 h-4" />
                    </button>
                </Popover.Trigger>
                <Popover.Content class="border w-64 mr-1 mt-2 text-sm" sideOffset={8}>
                    <div class="flex flex-col gap-3 p-1">
                        <div class="flex justify-between items-center gap-4">
                            <Label for="difficulty">Difficulty</Label>
                            <Select.Root type="single" bind:value={headerState.difficulty} disabled={!headerState.game.IsBettingPhase}>
                                <Select.Trigger class="w-[100px]">{headerState.difficulty}</Select.Trigger>
                                <Select.Content>
                                    <Select.Item value="Easy">Easy</Select.Item>
                                    <Select.Item value="Medium">Medium</Select.Item>
                                    <Select.Item value="Hard" disabled>Hard</Select.Item>
                                </Select.Content>
                            </Select.Root>
                        </div>
                        <div class="flex justify-between items-center gap-4">
                            <Label for="bot-speed">Bot Speed</Label>
                            <Input type="number" bind:value={headerState.botSpeed} class="w-[100px]" />
                        </div>
                        <div class="flex justify-between items-center gap-4">
                            <Label for="hidden-mode">Hidden Mode</Label>
                            <Switch bind:checked={headerState.hiddenMode} />
                        </div>
                        <div class="flex justify-between items-center gap-4">
                            <Label for="bots">Bots</Label>
                            <Switch bind:checked={headerState.game.TurnOnBots} />
                        </div>
                        <Separator />
                        <div class="flex justify-between items-center gap-4">
                            <Label for="light-mode">Light Mode</Label>
                            <Switch bind:checked={headerState.isLightMode} onclick={toggleMode} />
                        </div>
                    </div>
                </Popover.Content>
            </Popover.Root>
        {/if}
    </div>

    <!-- Right section -->
    <div class="flex items-center gap-3">
        <button
            disabled
            class="p-1.5 rounded-md text-muted-foreground opacity-30 cursor-not-allowed"
            title="Notifications (coming soon)"
        >
            <Bell class="w-4 h-4" />
        </button>

        <a
            href="/user"
            class="flex items-center gap-1.5 text-sm text-foreground hover:text-accent transition-colors no-underline"
        >
            <User class="w-4 h-4 text-muted-foreground" />
            {#if headerState.loggedIn}
                <span>{headerState.username}</span>
            {:else}
                <span class="text-muted-foreground">login</span>
            {/if}
        </a>
    </div>
</header>
