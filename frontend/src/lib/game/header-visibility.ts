export type HeaderVisibilityInput = {
    roomParam: string | null
    isOnline: boolean
    isJoiningRoom: boolean
    lobbyRoomId: string
}

export function shouldShowSiteHeader(input: HeaderVisibilityInput): boolean {
    return !(input.roomParam || input.isOnline || input.isJoiningRoom || input.lobbyRoomId)
}