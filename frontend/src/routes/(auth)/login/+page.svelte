<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { goto } from "$app/navigation";

    let username = $state("")
    let password = $state("")
    let error = $state("")
    let loading = $state(false)

    function getApiUrl(): string {
        if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
            return "http://127.0.0.1:3000"
        }
        return ""
    }

    async function login() {
        if (!username.trim() || !password) return
        loading = true; error = ""
        try {
            const res = await fetch(`${getApiUrl()}/api/auth/login`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username: username.trim(), password }),
            })
            const d = await res.json()
            if (!d.ok) { error = d.error || "Login failed"; return }
            document.cookie = `session=${d.token}; path=/; max-age=${30*24*60*60}; SameSite=Lax`
            goto("/")
        } catch (e) {
            error = "Failed to connect to server"
        } finally {
            loading = false
        }
    }
</script>

<div class="flex justify-center items-center w-full h-screen">
    <Card.Root class="sm:w-[400px] w-[70%]">
      <Card.Header>
        <Card.Title>Log In</Card.Title>
        <Card.Description>Sign in to your account</Card.Description>
      </Card.Header>
      <form onsubmit={(e) => { e.preventDefault(); login() }}>
          <Card.Content>
              <div class="mb-4">
                  <label class="text-sm font-medium mb-1 block" for="username">Username</label>
                  <Input id="username" bind:value={username} />
              </div>
              <div class="mb-8">
                  <label class="text-sm font-medium mb-1 block" for="password">Password</label>
                  <Input id="password" type="password" bind:value={password} />
              </div>
              {#if error}
                  <p class="text-sm text-destructive mb-2">{error}</p>
              {/if}
          </Card.Content>
          <Card.Footer class="flex justify-end">
               <div class="flex justify-between w-full">
                    <a href="/signup" class="text-xs italic underline text-foreground/60 h-full flex items-end pb-2">
                        Don't have an account?
                    </a>
                    <Button type="submit" disabled={loading} class="w-[80px]">
                        {loading ? "..." : "Login"}
                    </Button>
                </div>
          </Card.Footer>
        </form>
    </Card.Root>
</div>
