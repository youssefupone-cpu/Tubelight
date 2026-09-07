import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { playlist_items, playlists } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/feed/watch-later")({
	component: WatchLaterFeed,
});

export function WatchLaterFeed() {
	const pl = useQuery({
		queryKey: ["playlists"],
		queryFn: playlists,
	});
	const wlId = pl.data?.find((p) => p.is_watch_later)?.id ?? null;
	const items = useQuery({
		queryKey: ["playlist-items", wlId],
		queryFn: wlId ? () => playlist_items(wlId) : () => Promise.resolve([]),
		enabled: !!wlId,
	});

	if (pl.isPending || items.isPending)
		return <div className="p-4">Loading…</div>;
	if (pl.error)
		return <div className="p-4 text-red-400">{String(pl.error)}</div>;
	return <VideoGrid items={items.data ?? []} />;
}
