<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";

    let error = $state("");

    onMount(() => {
        const token = page.url.searchParams.get("token");
        const err = page.url.searchParams.get("error");

        if (err) {
            error = decodeURIComponent(err);
            return;
        }

        if (token) {
            // Set the session cookie and redirect to home
            document.cookie = `session=${token}; path=/; max-age=${30 * 24 * 60 * 60}; SameSite=Lax`;
            goto("/");
        } else {
            error = "No session token received";
        }
    });
</script>

<div class="flex justify-center items-center w-full h-screen">
    {#if error}
        <div class="text-center">
            <p class="text-destructive text-lg mb-4">Sign-in failed</p>
            <p class="text-muted-foreground text-sm mb-4">{error}</p>
            <a href="/login" class="text-sm underline">Back to login</a>
        </div>
    {:else}
        <div class="text-center">
            <div class="animate-spin text-2xl mb-4">⟳</div>
            <p class="text-muted-foreground">Signing you in...</p>
        </div>
    {/if}
</div>
