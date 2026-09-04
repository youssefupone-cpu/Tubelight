import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Link, useSearch } from "@tanstack/react-router";
import { playlists } from "@yoube/contracts";
import { SubscriptionsFeed } from "./feed.subscriptions";
import { HistoryFeed } from "./feed.history";
import { LikedFeed } from "./feed.liked";
import { WatchLaterFeed } from "./feed.watch-later";
import { cn } from "../lib/utils";

export const Route = createFileRoute("/library")({
	validateSearch: (search) => ({
		tab: (search as { tab?: string }).tab ?? "subscriptions",
	}),
	component: Library,
});

const TABS = [
	{ id: "subscriptions", label: "Subscriptions" },
	{ id: "history", label: "History" },
	{ id: "liked", label: "Liked" },
	{ id: "watch-later", label: "Watch later" },
	{ id: "playlists", label: "Playlists" },
	{ id: "downloads", label: "Downloads" },
	{ id: "settings", label: "Settings" },
] as const;

function PlaylistsView() {
	const q = useQuery({
		queryKey: ["playlists"],
		queryFn: playlists,
	});
	if (q.isPending) return <div className="p-4">Loading…</div>;
	if (q.error) return <div className="p-4 text-red-400">{String(q.error)}</div>;
	return (
		<div className="p-4">
			<h2 className="text-xl font-semibold mb-4">Playlists</h2>
			{q.data?.length === 0 && (
				<p className="text-neutral-400">No playlists yet</p>
			)}
			<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
				{q.data?.map((p) => (
					<div key={p.id} className="bg-neutral-800 rounded-xl p-4">
						<div className="font-medium">{p.title}</div>
						<div className="text-sm text-neutral-400">
							{p.count} video{p.count !== 1 ? "s" : ""}
						</div>
					</div>
				))}
			</div>
		</div>
	);
}

function Library() {
	const { tab } = useSearch({ from: "/library" });

	return (
		<div className="h-full flex flex-col">
			<nav className="border-b border-neutral-800">
				<div className="flex gap-4 px-4 overflow-x-auto">
					{TABS.map((t) => (
						<Link
							key={t.id}
							to="/library"
							search={{ tab: t.id }}
							className={cn(
								"px-4 py-3 text-sm border-b-2 whitespace-nowrap transition-colors",
								tab === t.id
									? "border-white"
									: "border-transparent hover:border-neutral-500",
							)}
						>
							{t.label}
						</Link>
					))}
				</div>
			</nav>
			<main className="flex-1 overflow-auto">
				{tab === "subscriptions" && <SubscriptionsFeed />}
				{tab === "history" && <HistoryFeed />}
				{tab === "liked" && <LikedFeed />}
				{tab === "watch-later" && <WatchLaterFeed />}
				{tab === "playlists" && <PlaylistsView />}
				{(tab === "downloads" || tab === "settings") && (
					<div className="p-4 text-neutral-400">Coming soon</div>
				)}
			</main>
		</div>
	);
}
