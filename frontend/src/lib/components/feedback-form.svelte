<script lang="ts">
	import { Label } from "$lib/components/ui/label/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Check } from "@lucide/svelte";

	let {
		matchId,
		playerId,
		onlineToken,
	}: {
		matchId: number | null;
		playerId: number;
		onlineToken: string;
	} = $props();

	let featureRequests = $state("");
	let bugReports = $state("");
	let submitting = $state(false);
	let submitted = $state(false);
	let error = $state("");

	async function submitFeedback() {
		if (submitting || submitted || !matchId) return;
		submitting = true;
		error = "";

		const API_URL =
			typeof window !== "undefined" &&
			(window.location.hostname === "localhost" ||
				window.location.hostname === "127.0.0.1")
				? "http://127.0.0.1:3000"
				: "https://bridge-club.duckdns.org";

		try {
			const res = await fetch(`${API_URL}/api/feedback`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
					"X-Session-Token": onlineToken,
				},
				body: JSON.stringify({
					matchId,
					playerId,
					featureRequests,
					bugReports,
				}),
			});
			const data = await res.json();
			if (data.ok) {
				submitted = true;
			} else {
				error = data.error ?? "Failed to submit feedback";
			}
		} catch (e) {
			error = "Network error";
		} finally {
			submitting = false;
		}
	}
</script>

<div class="flex flex-col gap-4 w-full pt-2">
	{#if submitted}
		<div class="flex items-center gap-2 text-sm text-[var(--green)]">
			<Check class="w-4 h-4" />
			<span>Thanks for your feedback!</span>
		</div>
	{:else}
		<div class="flex flex-col gap-1.5">
			<Label for="feature-requests" class="text-xs text-muted-foreground">
				Feature Requests
			</Label>
			<Textarea
				id="feature-requests"
				bind:value={featureRequests}
				placeholder="What features would you like to see?"
				class="min-h-[60px] text-sm"
				disabled={submitting}
			/>
		</div>
		<div class="flex flex-col gap-1.5">
			<Label for="bug-reports" class="text-xs text-muted-foreground">
				Bug Reports
			</Label>
			<Textarea
				id="bug-reports"
				bind:value={bugReports}
				placeholder="Found a bug? Tell us about it."
				class="min-h-[60px] text-sm"
				disabled={submitting}
			/>
		</div>
		{#if error}
			<p class="text-xs text-[var(--red)]">{error}</p>
		{/if}
		<Button
			size="sm"
			variant="outline"
			onclick={submitFeedback}
			disabled={submitting}
			class="self-end"
		>
			{submitting ? "Submitting..." : "Send Feedback"}
		</Button>
	{/if}
</div>
