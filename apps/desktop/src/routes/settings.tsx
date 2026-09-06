import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings")({
	component: SettingsPlaceholder,
});

function SettingsPlaceholder() {
	return (
		<div className="h-full overflow-auto p-4">
			<h1 className="text-xl font-semibold">Settings</h1>
			<p className="mt-2 text-sm text-neutral-400">
				Privacy controls (ad-block layers) land here in Phase 4.
			</p>
		</div>
	);
}
