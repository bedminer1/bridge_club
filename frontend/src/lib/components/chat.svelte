<script lang="ts">
    import { Card, CardContent } from "$lib/components/ui/card/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    interface ChatMsg { id: number; playerName: string; text: string; timestamp: number }

    let {
        roomId = "",
        lobbyPlayerId = "",
        chatMessages = $bindable<ChatMsg[]>([]),
        onSend = (_text: string) => {},
    } = $props()

    let chatText = $state("")
    let chatContainer: HTMLDivElement | undefined = $state(undefined)

    function fmtChatTime(ts: number): string {
        return new Date(ts).toLocaleTimeString("en-SG", { hour: "2-digit", minute: "2-digit", hour12: false })
    }

    function handleSend() {
        const text = chatText.trim()
        if (!text || !roomId || !lobbyPlayerId) return
        chatText = ""
        onSend(text)
    }

    function handleKey(e: KeyboardEvent) {
        if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend() }
    }

    // Auto-scroll to bottom on new messages
    $effect(() => {
        if (chatMessages.length > 0 && chatContainer) {
            setTimeout(() => { chatContainer!.scrollTop = chatContainer!.scrollHeight }, 50)
        }
    })
</script>

<div class="w-full xl:w-64 flex flex-col">
    <Card class="h-full flex flex-col">
        <CardContent class="flex flex-col gap-2 p-3 flex-1">
            <div class="text-xs font-medium text-muted-foreground">Chat</div>
            <div bind:this={chatContainer} class="flex-1 overflow-y-auto space-y-1 scrollbar-thin" style="min-height:120px">
                {#each chatMessages as msg, i (msg.id)}
                    <div>
                        {#if i === 0 || chatMessages[i-1].playerName !== msg.playerName}
                            <span class="text-xs font-semibold text-accent">{msg.playerName}</span>
                            <span class="text-xs text-muted-foreground tabular-nums">{fmtChatTime(msg.timestamp)}</span>
                        {/if}
                        <div class="text-sm text-foreground/90 break-words">{msg.text}</div>
                    </div>
                {:else}
                    <p class="text-xs text-muted-foreground text-center pt-8">No messages yet</p>
                {/each}
            </div>
            <div class="flex gap-2">
                <Input
                    bind:value={chatText}
                    onkeydown={handleKey}
                    placeholder="Chat..."
                    maxlength={500}
                    class="flex-1 h-8 text-xs"
                />
                <Button onclick={handleSend} size="sm" class="h-8 px-3 text-xs">Send</Button>
            </div>
        </CardContent>
    </Card>
</div>
