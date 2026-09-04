import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import {
	BellPlus,
	BellMinus,
	Heart,
	Plus,
	Check,
} from "lucide-react";
import type { VideoSummary } from "@yoube/contracts";
import {
	subscriptions,
	subscribe,
	unsubscribe,
	playlists,
	playlist_items,
	playlist_remove,
	like,
	unlike,
	watch_later,
} from "@yoube/contracts";
import { cn } from "../lib/utils";

function SubscribeBtn({
	channelId,
	channelTitle,
	thumbUrl,
}: {
	channelId: string;
	channelTitle: string;
	thumbUrl: string | null;
}) {
	const qc = useQueryClient();
	const subs = useQuery({ queryKey: ["subscriptions"], queryFn: subscriptions });
	const isSub = subs.data?.some((s) => s.id === channelId) ?? false;
	const mut = useMutation({
		mutationFn: isSub
			? () => unsubscribe(channelId)
			: () => subscribe(channelId, channelTitle, thumbUrl),
		onSuccess: () => qc.invalidateQueries({ queryKey: ["subscriptions"] }),
	});
	return (
		<button
			onClick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				mut.mutate();
			}}
			className={cn(
				"p-1 rounded hover:bg-neutral-700 transition-colors",
				isSub ? "text-white" : "text-neutral-400",
			)}
			title={isSub ? "Subscribed" : "Subscribe"}
		>
			{isSub ? <BellMinus size={16} /> : <BellPlus size={16} />}
		</button>
	);
}

function LikeBtn({ video }: { video: VideoSummary }) {
	const qc = useQueryClient();
	const pl = useQuery({ queryKey: ["playlists"], queryFn: playlists });
	const likedId = pl.data?.find((p) => p.is_liked)?.id ?? null;
	const items = useQuery({
		queryKey: ["playlist-items", likedId],
		queryFn: likedId ? () => playlist_items(likedId) : () => Promise.resolve([]),
		enabled: !!likedId,
	});
	const isLiked = items.data?.some((i) => i.id === video.id) ?? false;
	const mut = useMutation({
		mutationFn: isLiked ? () => unlike(video.id) : () => like(video),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ["playlists"] });
			qc.invalidateQueries({ queryKey: ["playlist-items"] });
		},
	});
	return (
		<button
			onClick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				mut.mutate();
			}}
			className={cn(
				"p-1 rounded hover:bg-neutral-700 transition-colors",
				isLiked ? "text-red-500" : "text-neutral-400",
			)}
			title={isLiked ? "Unlike" : "Like"}
		>
			<Heart size={16} fill={isLiked ? "currentColor" : "none"} />
		</button>
	);
}

function WatchLaterBtn({ video }: { video: VideoSummary }) {
	const qc = useQueryClient();
	const pl = useQuery({ queryKey: ["playlists"], queryFn: playlists });
	const wlId = pl.data?.find((p) => p.is_watch_later)?.id ?? null;
	const items = useQuery({
		queryKey: ["playlist-items", wlId],
		queryFn: wlId ? () => playlist_items(wlId) : () => Promise.resolve([]),
		enabled: !!wlId,
	});
	const isInWl = items.data?.some((i) => i.id === video.id) ?? false;
	const mut = useMutation({
		mutationFn: isInWl
			? () => (wlId ? playlist_remove(wlId, video.id) : Promise.resolve())
			: () => watch_later(video),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ["playlists"] });
			qc.invalidateQueries({ queryKey: ["playlist-items"] });
		},
	});
	return (
		<button
			onClick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				mut.mutate();
			}}
			className={cn(
				"p-1 rounded hover:bg-neutral-700 transition-colors",
				isInWl ? "text-white" : "text-neutral-400",
			)}
			title={isInWl ? "Remove from watch later" : "Save to watch later"}
		>
			{isInWl ? <Check size={16} /> : <Plus size={16} />}
		</button>
	);
}

export function VideoCard({ v }: { v: VideoSummary }) {
	return (
		<div className="group">
			<Link
				to="/watch"
				search={{ v: v.id }}
				className="block"
			>
				<div className="aspect-video bg-neutral-800 rounded-xl overflow-hidden">
					{v.thumbnail_url && (
						<img
							src={v.thumbnail_url}
							alt=""
							className="w-full h-full object-cover group-hover:scale-[1.02] transition-transform"
						/>
					)}
				</div>
				<div className="mt-2">
					<div className="font-medium line-clamp-2">{v.title}</div>
					<div className="text-sm text-neutral-400">{v.channel_title}</div>
				</div>
			</Link>
			<div className="flex items-center gap-1 mt-1">
				<SubscribeBtn
					channelId={v.channel_id}
					channelTitle={v.channel_title}
					thumbUrl={v.thumbnail_url ?? null}
				/>
				<LikeBtn video={v} />
				<WatchLaterBtn video={v} />
			</div>
		</div>
	);
}
