import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { channel, channel_videos } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/channel/$id")({ component: ChannelPage });
function ChannelPage() {
  const { id } = useParams({ from: "/channel/$id" });
  const ch = useQuery({ queryKey: ["channel", id], queryFn: () => channel(id) });
  const videos = useQuery({ queryKey: ["channel-videos", id], queryFn: () => channel_videos(id, 0) });
  if (ch.isPending) return <div className="p-4">Loading…</div>;
  return (
    <div className="h-full overflow-auto">
      <div className="p-4 border-b border-neutral-800">
        <div className="text-xl font-semibold">{ch.data?.title}</div>
        <div className="text-sm text-neutral-400 line-clamp-3">{ch.data?.description}</div>
      </div>
      <VideoGrid items={videos.data ?? []} />
    </div>
  );
}
