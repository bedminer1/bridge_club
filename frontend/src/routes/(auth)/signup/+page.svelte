<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { goto } from "$app/navigation";

    let username = $state("")
    let password = $state("")
    let error = $state("")
    let loading = $state(false)
    let googleLoading = $state(false)

    function getApiUrl(): string {
        if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
            return "http://127.0.0.1:3000"
        }
        return ""
    }

    async function signup() {
        if (!username.trim() || !password) return
        loading = true; error = ""
        try {
            const res = await fetch(`${getApiUrl()}/api/auth/signup`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username: username.trim(), password }),
            })
            const d = await res.json()
            if (!d.ok) { error = d.error || "Signup failed"; return }
            document.cookie = `session=${d.token}; path=/; max-age=${30*24*60*60}; SameSite=Lax`
            goto("/")
        } catch {
            error = "Failed to connect to server"
        } finally {
            loading = false
        }
    }

    async function signInWithGoogle() {
        googleLoading = true; error = ""
        try {
            const res = await fetch(`${getApiUrl()}/api/auth/google/login`)
            const d = await res.json()
            if (!d.ok || !d.url) { error = d.error || "Failed to start Google sign-in"; return }
            window.location.href = d.url
        } catch {
            error = "Failed to connect to server"
        } finally {
            googleLoading = false
        }
    }
</script>

<div class="flex justify-center items-center w-full h-screen">
    <Card.Root class="sm:w-[400px] w-[70%]">
      <Card.Header>
        <Card.Title>Sign Up</Card.Title>
        <Card.Description>Create a new account</Card.Description>
      </Card.Header>
      <form onsubmit={(e) => { e.preventDefault(); signup() }}>
          <Card.Content>
              <div class="mb-4">
                  <label class="text-sm font-medium mb-1 block" for="username">Username</label>
                  <Input id="username" bind:value={username} placeholder="Your unique name" />
              </div>
              <div class="mb-8">
                  <label class="text-sm font-medium mb-1 block" for="password">Password</label>
                  <Input id="password" type="password" bind:value={password} placeholder="A strong password" />
              </div>
              {#if error}
                  <p class="text-sm text-destructive mb-2">{error}</p>
              {/if}

              <div class="mb-4">
                  <button
                      onclick={signInWithGoogle}
                      disabled={googleLoading}
                      class="w-full flex items-center justify-center gap-2 border border-input rounded-md px-4 py-2 text-sm hover:bg-accent transition-colors"
                  >
                      {#if googleLoading}
                          <span class="animate-spin">⟳</span>
                      {:else}
                          <svg class="w-5 h-5" viewBox="0 0 24 24">
                              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/>
                              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                          </svg>
                      {/if}
                      <span>Sign up with Google</span>
                  </button>
              </div>

              <div class="relative mb-4">
                  <div class="absolute inset-0 flex items-center"><span class="w-full border-t"></span></div>
                  <div class="relative flex justify-center text-xs uppercase"><span class="bg-card px-2 text-muted-foreground">or</span></div>
              </div>
          </Card.Content>
          <Card.Footer class="flex justify-end">
              <div class="flex justify-between w-full">
                  <a href="/login" class="text-xs italic underline text-foreground/60 h-full flex items-end pb-2">
                    Already have an account?
                  </a>
                  <Button type="submit" disabled={loading} class="w-[80px]">
                      {loading ? "..." : "Signup"}
                  </Button>
              </div>
          </Card.Footer>
        </form>
        <p class="text-xs text-muted-foreground text-center mt-4">
            By signing up, you agree to our
            <a href="/about#terms" class="underline hover:text-foreground">Terms</a> and
            <a href="/about#privacy" class="underline hover:text-foreground">Privacy Policy</a>
        </p>
    </Card.Root>
</div>
