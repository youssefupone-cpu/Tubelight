import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { history } from "@yoube/contracts";
import type { VideoSummary } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/feed/history")({
	component: HistoryFeed,
});

export function HistoryFeed() {
	const q = useQuery({
		queryKey: ["history", 0],
		queryFn: () => history(0),
	});
	if (q.isPending) return <div className="p-4">Loading…</div>;
	if (q.error) return <div className="p-4 text-red-400">{String(q.error)}</div>;

	const items: VideoSummary[] = (q.data ?? []).map((h) => ({
		id: h.video_id,
		title: h.title,
		channel_id: "",
		channel_title: h.channel_title ?? "",
		duration_s: h.duration_s,
		thumbnail_url: h.thumb_url,
	}));

	return <VideoGrid items={items} />;
}
