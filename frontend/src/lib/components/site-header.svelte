<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";

    import { Gamepad2, Crown, Info, Settings, Bell, User } from "@lucide/svelte";
    import { page } from "$app/state";

    import { headerState } from "$lib/game/header-state.svelte";
</script>

<header
    class="fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-6 py-3 border-b border-border bg-background/80 backdrop-blur-sm"
>
    <!-- Left section -->
    <div class="flex items-center">
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

    </div>

    <!-- Right section -->
    <div class="flex items-center gap-3">
        <div class="hidden sm:block">
        <Popover.Root>
            <Popover.Trigger class="p-1.5 rounded-md text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors">
                <Bell class="w-4 h-4" />
            </Popover.Trigger>
            <Popover.Content class="text-xs mr-2 mt-2" sideOffset={8}>
                Notifications Cleared
            </Popover.Content>
        </Popover.Root>
        </div>

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
