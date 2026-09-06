import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { channel, channel_videos } from "@yoube/contracts";
import {
	ChannelAvatar,
	SubscribeBtn,
	VideoGridSkeleton,
} from "../components/VideoCard";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/channel/$id")({
	component: ChannelPage,
});
function ChannelPage() {
	const { id } = useParams({ from: "/channel/$id" });
	const ch = useQuery({
		queryKey: ["channel", id],
		queryFn: () => channel(id),
	});
	const videos = useQuery({
		queryKey: ["channel-videos", id],
		queryFn: () => channel_videos(id, 0),
	});
	if (ch.isPending) return <VideoGridSkeleton count={8} />;
	if (ch.error)
		return <div className="p-4 text-red-400">{String(ch.error)}</div>;
	return (
		<div className="h-full overflow-auto">
			<div className="px-4 py-5 border-b border-neutral-800/80">
				<div className="flex items-center gap-4">
					{ch.data?.thumb_url ? (
						<img
							src={ch.data.thumb_url}
							alt=""
							className="w-16 h-16 rounded-full object-cover border border-white/10"
						/>
					) : (
						<ChannelAvatar name={ch.data?.title ?? "?"} size="lg" />
					)}
					<div className="min-w-0 flex-1">
						<div dir="auto" className="text-xl font-bold truncate">
							{ch.data?.title}
						</div>
						<div
							dir="auto"
							className="text-sm text-neutral-400 line-clamp-2 mt-0.5"
						>
							{ch.data?.description}
						</div>
					</div>
					{ch.data && (
						<SubscribeBtn
							channelId={ch.data.id}
							channelTitle={ch.data.title}
							thumbUrl={ch.data.thumb_url ?? null}
						/>
					)}
				</div>
			</div>
			<VideoGrid items={videos.data ?? []} />
		</div>
	);
}
