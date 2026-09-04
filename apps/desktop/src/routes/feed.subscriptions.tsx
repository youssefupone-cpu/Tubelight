import { useQuery, useQueries } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { subscriptions, channel_videos } from "@yoube/contracts";
import type { VideoSummary } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/feed/subscriptions")({
	component: SubscriptionsFeed,
});

export function SubscriptionsFeed() {
	const subs = useQuery({
		queryKey: ["subscriptions"],
		queryFn: subscriptions,
	});

	const videoQueries = useQueries({
		queries: (subs.data ?? []).map((sub) => ({
			queryKey: ["channel-videos", sub.id, 0],
			queryFn: () => channel_videos(sub.id, 0),
		})),
	});

	const allVideos: VideoSummary[] = videoQueries
		.flatMap((q) => (q.data as VideoSummary[] | undefined) ?? [])
		.sort((a, b) => {
			const da = a.upload_date ?? "";
			const db = b.upload_date ?? "";
			return db.localeCompare(da);
		});

	if (subs.isPending) return <div className="p-4">Loading…</div>;
	if (subs.error) return <div className="p-4 text-red-400">{String(subs.error)}</div>;
	return <VideoGrid items={allVideos} />;
}
