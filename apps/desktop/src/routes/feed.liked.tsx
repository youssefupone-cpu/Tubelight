import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { playlists, playlist_items } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/feed/liked")({
	component: LikedFeed,
});

export function LikedFeed() {
	const pl = useQuery({
		queryKey: ["playlists"],
		queryFn: playlists,
	});
	const likedId = pl.data?.find((p) => p.is_liked)?.id ?? null;
	const items = useQuery({
		queryKey: ["playlist-items", likedId],
		queryFn: likedId ? () => playlist_items(likedId) : () => Promise.resolve([]),
		enabled: !!likedId,
	});

	if (pl.isPending || items.isPending) return <div className="p-4">Loading…</div>;
	if (pl.error) return <div className="p-4 text-red-400">{String(pl.error)}</div>;
	return <VideoGrid items={items.data ?? []} />;
}
