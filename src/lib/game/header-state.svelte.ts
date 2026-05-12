/** @file Shared reactive state for site-header features.
 *  Page components sync to this singleton; site-header reads it reactively. */

import type { Game } from "./types"

class HeaderState {
    game = $state<Game | null>(null)
    difficulty = $state("Medium")
    botSpeed = $state(2)
    hiddenMode = $state(true)
    isLightMode = $state(false)
    username = $state("")
    loggedIn = $state(false)
}

export const headerState = new HeaderState()
