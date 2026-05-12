import "crypto"

export async function hashPassword(password: string): Promise<string> {
	const encoder = new TextEncoder()
	const data = encoder.encode(password)
	const hashBuffer = await crypto.subtle.digest("SHA-256", data)
	return Buffer.from(hashBuffer).toString("base64")
}