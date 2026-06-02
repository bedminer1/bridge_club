/** @file WebSocket client for real-time game/lobby/chat communication.
 *  Replaces HTTP polling with a single persistent WS connection.
 *  Exports a singleton client that emits typed events. */

type WsEventCallback = (data: any) => void

type WsEventMap = {
    "auth:ok": { userId: number; username: string }
    "auth:error": { error: string }
    "lobby:created": { roomId: string; playerId: string; seatIndex: number }
    "lobby:joined": { roomId: string; playerId: string; seatIndex: number }
    "lobby:update": { players: Array<{ name: string; seatIndex: number; isBot: boolean }>; hiddenMode: boolean }
    "lobby:started": { roomId: string }
    "lobby:left": {}
    "game:state": { state: any; roomId: string }
    "chat:message": { id: number; playerName: string; text: string; timestamp: number }
    "chat:sent": { ok: boolean }
    "lobby:hidden_toggled": { enabled: boolean }
    "error": { error: string }
}

type WsEventType = keyof WsEventMap

function getWsUrl(): string {
    if (typeof window !== 'undefined' && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')) {
        return "ws://127.0.0.1:3000/ws"
    }
    return "wss://bridge-club.duckdns.org/ws"
}

class WsClient {
    private ws: WebSocket | null = null
    private listeners = new Map<string, Set<WsEventCallback>>()
    private _connected = false
    private _connectionPromise: Promise<void> | null = null
    private _connectionResolve: (() => void) | null = null
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null
    private pendingMessages: string[] = []
    private _token: string = ""

    /** Create and connect. Call after getting an auth token. */
    connect(token: string): void {
        if (this.ws && this._connected) return
        this._token = token
        this._connectionPromise = new Promise((resolve) => {
            this._connectionResolve = resolve
        })

        const url = getWsUrl()
        console.log("[WS] Connecting to", url)
        this.ws = new WebSocket(url)

        this.ws.onopen = () => {
            console.log("[WS] Connected")
            this._connected = true
            // Send auth
            this.send({ type: "auth", token })
            // Flush pending
            for (const msg of this.pendingMessages) {
                this.ws?.send(msg)
            }
            this.pendingMessages = []
        }

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data)
                const type: string = data.type || "unknown"
                this.emit(type as WsEventType, data)
            } catch (e) {
                console.warn("[WS] Failed to parse message:", e)
            }
        }

        this.ws.onclose = () => {
            console.log("[WS] Disconnected")
            this._connected = false
            this._connectionPromise = null
            this._connectionResolve = null
            // Auto-reconnect after 3s
            if (this.reconnectTimer) clearTimeout(this.reconnectTimer)
            this.reconnectTimer = setTimeout(() => {
                if (this._token) this.connect(this._token)
            }, 3000)
        }

        this.ws.onerror = (err) => {
            console.error("[WS] Error:", err)
        }
    }

    /** Disconnect and stop reconnecting. */
    disconnect(): void {
        if (this.reconnectTimer) clearTimeout(this.reconnectTimer)
        this._token = ""
        this.pendingMessages = []
        if (this.ws) {
            this.ws.onclose = null // prevent reconnect
            this.ws.close()
            this.ws = null
        }
        this._connected = false
        this._connectionPromise = null
        this._connectionResolve = null
    }

    /** Send a JSON message. Queues if not yet connected. */
    send(data: Record<string, unknown>): void {
        const msg = JSON.stringify(data)
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(msg)
        } else {
            this.pendingMessages.push(msg)
        }
    }

    /** Wait for connection + auth to complete. */
    async waitForConnection(): Promise<void> {
        if (this._connected) return
        if (this._connectionPromise) return this._connectionPromise
        // If not even trying to connect, create a promise that resolves on first message
        return new Promise((resolve) => {
            const check = () => {
                if (this._connected) { resolve(); return }
                setTimeout(check, 100)
            }
            check()
        })
    }

    // ── Event system ───────────────────────────────────────────

    on<T extends WsEventType>(type: T, callback: (data: WsEventMap[T]) => void): () => void {
        if (!this.listeners.has(type)) {
            this.listeners.set(type, new Set())
        }
        this.listeners.get(type)!.add(callback as WsEventCallback)
        // Return unsubscribe function
        return () => {
            this.listeners.get(type)?.delete(callback as WsEventCallback)
        }
    }

    off<T extends WsEventType>(type: T, callback: (data: WsEventMap[T]) => void): void {
        this.listeners.get(type)?.delete(callback as WsEventCallback)
    }

    private emit(type: WsEventType, data: any): void {
        // Resolve connection promise on first auth response
        if ((type === "auth:ok" || type === "auth:error") && this._connectionResolve) {
            this._connectionResolve()
            this._connectionResolve = null
        }
        const typeListeners = this.listeners.get(type)
        if (typeListeners) {
            for (const cb of typeListeners) {
                try { cb(data) } catch (e) { console.error("[WS] Listener error:", e) }
            }
        }
        // Also emit to "error" listeners on error type
        if (type === "error") {
            const errListeners = this.listeners.get("error")
            if (errListeners) {
                for (const cb of errListeners) {
                    try { cb(data) } catch (e) { console.error("[WS] Listener error:", e) }
                }
            }
        }
    }

    get connected(): boolean { return this._connected }

    // ── High-level actions ─────────────────────────────────────

    /** Create a new multiplayer lobby room. */
    createLobby(): void {
        this.send({ type: "lobby:create" })
    }

    /** Join an existing lobby room. */
    joinLobby(roomId: string): void {
        this.send({ type: "lobby:join", roomId })
    }

    /** Leave the current room. */
    leaveLobby(): void {
        this.send({ type: "lobby:leave", playerId: "" })
    }

    /** Start the game in the current lobby. */
    startGame(hiddenMode: boolean): void {
        this.send({ type: "lobby:start", hiddenMode })
    }

    /** Toggle hidden mode. */
    toggleHidden(enabled: boolean): void {
        this.send({ type: "lobby:toggle_hidden", enabled })
    }

    /** Send a game action (bid, play card, select partner). */
    gameAction(actionType: string, call?: any, card?: any): void {
        this.send({ type: "game:action", actionType, call, card })
    }

    /** Send a chat message. */
    sendChat(playerId: string, text: string): void {
        this.send({ type: "chat:send", playerId, text })
    }
}

/** Singleton WebSocket client instance. */
export const wsClient = new WsClient()
