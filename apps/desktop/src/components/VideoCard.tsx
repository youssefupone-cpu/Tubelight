import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type { VideoSummary } from "@yoube/contracts";
import {
	like,
	playlist_items,
	playlist_remove,
	playlists,
	subscribe,
	subscriptions,
	unlike,
	unsubscribe,
	watch_later,
} from "@yoube/contracts";
import { BellMinus, BellPlus, Check, Heart, Plus } from "lucide-react";
import { formatDuration, formatViews, initialOf, timeAgo } from "../lib/format";
import { cn } from "../lib/utils";

export function SubscribeBtn({
	channelId,
	channelTitle,
	thumbUrl,
}: {
	channelId: string;
	channelTitle: string;
	thumbUrl: string | null;
}) {
	const qc = useQueryClient();
	const subs = useQuery({
		queryKey: ["subscriptions"],
		queryFn: subscriptions,
	});
	const isSub = subs.data?.some((s) => s.id === channelId) ?? false;
	const mut = useMutation({
		mutationFn: isSub
			? () => unsubscribe(channelId)
			: () => subscribe(channelId, channelTitle, thumbUrl),
		onSuccess: () => qc.invalidateQueries({ queryKey: ["subscriptions"] }),
	});
	return (
		<button
			type="button"
			onClick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				mut.mutate();
			}}
			className={cn(
				"p-1.5 rounded-full hover:bg-neutral-700/60 transition-colors",
				isSub ? "text-white" : "text-neutral-500 hover:text-neutral-200",
			)}
			title={isSub ? "Subscribed" : "Subscribe"}
		>
			{isSub ? <BellMinus size={15} /> : <BellPlus size={15} />}
		</button>
	);
}

export function LikeBtn({ video }: { video: VideoSummary }) {
	const qc = useQueryClient();
	const pl = useQuery({ queryKey: ["playlists"], queryFn: playlists });
	const likedId = pl.data?.find((p) => p.is_liked)?.id ?? null;
	const items = useQuery({
		queryKey: ["playlist-items", likedId],
		queryFn: likedId
			? () => playlist_items(likedId)
			: () => Promise.resolve([]),
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
			type="button"
			onClick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				mut.mutate();
			}}
			className={cn(
				"p-1.5 rounded-full hover:bg-neutral-700/60 transition-colors",
				isLiked ? "text-red-500" : "text-neutral-500 hover:text-neutral-200",
			)}
			title={isLiked ? "Unlike" : "Like"}
		>
			<Heart size={15} fill={isLiked ? "currentColor" : "none"} />
		</button>
	);
}

export function WatchLaterBtn({ video }: { video: VideoSummary }) {
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
			type="button"
			onClick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				mut.mutate();
			}}
			className={cn(
				"p-1.5 rounded-full hover:bg-neutral-700/60 transition-colors",
				isInWl ? "text-white" : "text-neutral-500 hover:text-neutral-200",
			)}
			title={isInWl ? "Remove from watch later" : "Save to watch later"}
		>
			{isInWl ? <Check size={15} /> : <Plus size={15} />}
		</button>
	);
}

export function ChannelAvatar({
	name,
	size = "md",
}: {
	name: string;
	size?: "sm" | "md" | "lg";
}) {
	const dims =
		size === "lg"
			? "w-10 h-10 text-base"
			: size === "sm"
				? "w-7 h-7 text-xs"
				: "w-[34px] h-[34px] text-[13px]";
	return (
		<div
			className={cn(
				"rounded-full bg-neutral-700 flex-shrink-0 flex items-center justify-center font-semibold text-neutral-200",
				dims,
			)}
			aria-hidden
		>
			{initialOf(name)}
		</div>
	);
}

export function VideoMeta({
	v,
	className,
}: { v: VideoSummary; className?: string }) {
	const bits = [formatViews(v.view_count), timeAgo(v.upload_date)].filter(
		Boolean,
	);
	return (
		<div dir="auto" className={cn("text-xs text-neutral-400", className)}>
			{v.channel_title}
			{bits.length > 0 && <span> · {bits.join(" · ")}</span>}
		</div>
	);
}

export function VideoCard({ v }: { v: VideoSummary }) {
	const duration = formatDuration(v.duration_s);
	return (
		<div className="group">
			<Link to="/watch" search={{ v: v.id }} className="block">
				<div className="aspect-video bg-neutral-800 rounded-xl overflow-hidden relative border border-white/[0.04]">
					{v.thumbnail_url && (
						<img
							src={v.thumbnail_url}
							alt=""
							loading="lazy"
							className="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-300"
						/>
					)}
					{duration && (
						<span className="absolute right-2 bottom-2 bg-black/85 text-[11px] font-medium px-1.5 py-0.5 rounded-md">
							{duration}
						</span>
					)}
				</div>
				<div className="mt-2 flex gap-2.5 px-0.5">
					<ChannelAvatar name={v.channel_title} />
					<div className="min-w-0">
						<div
							dir="auto"
							className="text-sm font-semibold leading-snug line-clamp-2"
						>
							{v.title}
						</div>
						<VideoMeta v={v} className="mt-1" />
					</div>
				</div>
			</Link>
			<div className="flex items-center gap-0.5 mt-0.5 pl-11 opacity-60 hover:opacity-100 focus-within:opacity-100 transition-opacity">
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

export function VideoCardSkeleton() {
	return (
		<div aria-hidden>
			<div className="aspect-video rounded-xl yoube-skeleton" />
			<div className="mt-2 flex gap-2.5 px-0.5">
				<div className="w-[34px] h-[34px] rounded-full yoube-skeleton flex-shrink-0" />
				<div className="flex-1">
					<div className="h-3.5 rounded-md yoube-skeleton w-[90%]" />
					<div className="h-3 rounded-md yoube-skeleton w-[60%] mt-2" />
				</div>
			</div>
		</div>
	);
}

export function VideoGridSkeleton({ count = 8 }: { count?: number }) {
	return (
		<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-x-4 gap-y-6 p-4">
			{Array.from({ length: count }, (_, i) => (
				// biome-ignore lint/suspicious/noArrayIndexKey: static skeleton placeholders, order never changes
				<VideoCardSkeleton key={i} />
			))}
		</div>
	);
}
