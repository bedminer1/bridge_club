/**
 * Web Audio API sound synthesizer for Bridge Club.
 * No external audio files needed — all sounds are synthesized.
 * Uses AudioContext lazy-init on first user interaction.
 */

let ctx: AudioContext | null = null

function getContext(): AudioContext {
    if (!ctx) {
        ctx = new AudioContext()
    }
    if (ctx.state === "suspended") {
        ctx.resume()
    }
    return ctx
}

/** Short percussive burst — card being placed on the table */
export function playCardPlace(): void {
    const c = getContext()
    const t = c.currentTime

    // Noise burst with fast decay
    const bufferSize = c.sampleRate * 0.06
    const buffer = c.createBuffer(1, bufferSize, c.sampleRate)
    const data = buffer.getChannelData(0)
    for (let i = 0; i < bufferSize; i++) {
        data[i] = (Math.random() * 2 - 1) * (1 - i / bufferSize)
    }
    const noise = c.createBufferSource()
    noise.buffer = buffer

    // Bandpass filter to shape the "card slap" tone
    const filter = c.createBiquadFilter()
    filter.type = "bandpass"
    filter.frequency.value = 2000
    filter.Q.value = 0.5

    const gain = c.createGain()
    gain.gain.setValueAtTime(0.4, t)
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.06)

    noise.connect(filter).connect(gain).connect(c.destination)
    noise.start(t)
    noise.stop(t + 0.06)
}

/** Slightly softer tap — card being dealt / slid across table */
export function playCardDeal(): void {
    const c = getContext()
    const t = c.currentTime

    // Longer, softer noise
    const bufferSize = c.sampleRate * 0.12
    const buffer = c.createBuffer(1, bufferSize, c.sampleRate)
    const data = buffer.getChannelData(0)
    for (let i = 0; i < bufferSize; i++) {
        const env = 1 - i / bufferSize
        data[i] = (Math.random() * 2 - 1) * env * env
    }
    const noise = c.createBufferSource()
    noise.buffer = buffer

    const filter = c.createBiquadFilter()
    filter.type = "lowpass"
    filter.frequency.value = 800

    const gain = c.createGain()
    gain.gain.setValueAtTime(0.3, t)
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.12)

    noise.connect(filter).connect(gain).connect(c.destination)
    noise.start(t)
    noise.stop(t + 0.12)
}

/** Rising "ding" sound — trick / set won */
export function playTrickWon(): void {
    const c = getContext()
    const t = c.currentTime

    const osc = c.createOscillator()
    osc.type = "sine"
    osc.frequency.setValueAtTime(523, t)       // C5
    osc.frequency.exponentialRampToValueAtTime(1047, t + 0.1) // C6

    const gain = c.createGain()
    gain.gain.setValueAtTime(0.25, t)
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.3)

    osc.connect(gain).connect(c.destination)
    osc.start(t)
    osc.stop(t + 0.3)
}

/** Ascending arpeggio — game won */
export function playGameWon(): void {
    const c = getContext()
    const t = c.currentTime

    const notes = [523, 659, 784, 1047] // C5, E5, G5, C6
    notes.forEach((freq, i) => {
        const osc = c.createOscillator()
        osc.type = "sine"
        osc.frequency.value = freq

        const gain = c.createGain()
        const start = t + i * 0.12
        gain.gain.setValueAtTime(0, start)
        gain.gain.linearRampToValueAtTime(0.3, start + 0.02)
        gain.gain.exponentialRampToValueAtTime(0.001, start + 0.3)

        osc.connect(gain).connect(c.destination)
        osc.start(start)
        osc.stop(start + 0.3)
    })
}

/** Descending tone — game lost */
export function playGameLost(): void {
    const c = getContext()
    const t = c.currentTime

    const notes = [523, 440, 349, 262] // C5, A4, F4, C4
    notes.forEach((freq, i) => {
        const osc = c.createOscillator()
        osc.type = "triangle"
        osc.frequency.value = freq

        const gain = c.createGain()
        const start = t + i * 0.15
        gain.gain.setValueAtTime(0, start)
        gain.gain.linearRampToValueAtTime(0.2, start + 0.02)
        gain.gain.exponentialRampToValueAtTime(0.001, start + 0.4)

        osc.connect(gain).connect(c.destination)
        osc.start(start)
        osc.stop(start + 0.4)
    })
}

/** Soft click — for buttons / ui actions */
export function playUIClick(): void {
    const c = getContext()
    const t = c.currentTime

    const osc = c.createOscillator()
    osc.type = "sine"
    osc.frequency.value = 800

    const gain = c.createGain()
    gain.gain.setValueAtTime(0.1, t)
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.03)

    osc.connect(gain).connect(c.destination)
    osc.start(t)
    osc.stop(t + 0.03)
}
