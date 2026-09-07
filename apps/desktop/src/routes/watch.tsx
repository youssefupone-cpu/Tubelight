import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import {
	filter_segments_for,
	get_video,
	related,
	subscribe,
	subscriptions,
	unsubscribe,
} from "@yoube/contracts";
import { Download } from "lucide-react";
import { useState } from "react";
import { FormatPicker } from "../components/FormatPicker";
import { Player } from "../components/Player";
import {
	ChannelAvatar,
	LikeBtn,
	VideoCard,
	WatchLaterBtn,
} from "../components/VideoCard";
import { DEFAULT_SKIP_CATEGORIES, useSetting } from "../hooks/useSettings";
import { resolveMediaSrc } from "../lib/media";
import { cn } from "../lib/utils";

export const Route = createFileRoute("/watch")({
	validateSearch: (search) => ({
		v: (search as { v?: string }).v ?? "",
	}),
	component: Watch,
});

function Watch() {
	const { v } = useSearch({ from: "/watch" });
	const [pickerOpen, setPickerOpen] = useState(false);
	const [l3Enabled] = useSetting("l3_enabled", true);
	const [skipCats] = useSetting<readonly string[]>(
		"l3_categories",
		DEFAULT_SKIP_CATEGORIES,
	);
	const vid = useQuery({
		queryKey: ["video", v],
		queryFn: () => get_video(v),
		enabled: !!v,
	});
	const rel = useQuery({
		queryKey: ["related", v],
		queryFn: () => related(v),
		enabled: !!v,
	});
	const segments = useQuery({
		queryKey: ["segments", v],
		queryFn: () => filter_segments_for(v),
		enabled: !!v && l3Enabled,
		staleTime: 1000 * 60 * 60,
	});
	const qc = useQueryClient();
	const subs = useQuery({
		queryKey: ["subscriptions"],
		queryFn: subscriptions,
	});
	// Null-safe until the guards below: hooks must run unconditionally.
	const summary = vid.data?.summary;
	const isSub = subs.data?.some((s) => s.id === summary?.channel_id) ?? false;
	const subMut = useMutation({
		mutationFn: () => {
			if (!summary) return Promise.resolve();
			return isSub
				? unsubscribe(summary.channel_id)
				: subscribe(
						summary.channel_id,
						summary.channel_title,
						summary.thumbnail_url ?? null,
					);
		},
		onSuccess: () => qc.invalidateQueries({ queryKey: ["subscriptions"] }),
	});
	if (vid.isPending) return <div className="p-4">Loading…</div>;
	if (vid.error)
		return <div className="p-4 text-red-400">{String(vid.error)}</div>;
	const best =
		vid.data.formats.find(
			(f) => (f.vcodec ?? "none") !== "none" && (f.acodec ?? "none") !== "none",
		) ?? vid.data.formats[0];
	return (
		<div className="h-full overflow-auto p-4 grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-6">
			<div>
				<div className="aspect-video bg-black rounded-xl overflow-hidden border border-white/[0.06]">
					<Player
						src={resolveMediaSrc(best.url ?? "")}
						poster={vid.data.summary.thumbnail_url ?? undefined}
						segments={segments.data ?? []}
						skipEnabled={l3Enabled}
						skipCategories={skipCats}
					/>
				</div>
				<h1 dir="auto" className="text-xl font-bold mt-3">
					{vid.data.summary.title}
				</h1>
				<div className="mt-3 flex flex-wrap items-center gap-3">
					<div className="flex items-center gap-2.5">
						<ChannelAvatar name={vid.data.summary.channel_title} />
						<div dir="auto" className="text-sm font-semibold leading-tight">
							{vid.data.summary.channel_title}
							{(segments.data?.length ?? 0) > 0 && (
								<span className="ml-2 rounded-full bg-neutral-800 px-2 py-0.5 text-[11px] font-normal text-neutral-400">
									{segments.data?.length} skips
								</span>
							)}
						</div>
						<button
							type="button"
							onClick={() => subMut.mutate()}
							className={cn(
								"ml-2 rounded-full px-4 py-1.5 text-sm font-semibold transition-colors",
								isSub
									? "bg-neutral-800 text-white hover:bg-neutral-700"
									: "bg-white text-black hover:bg-neutral-200",
							)}
						>
							{isSub ? "Subscribed" : "Subscribe"}
						</button>
					</div>
					<div className="flex items-center gap-1.5 ml-auto">
						<span className="flex items-center rounded-full bg-neutral-800 px-1">
							<LikeBtn video={vid.data.summary} />
						</span>
						<span className="flex items-center rounded-full bg-neutral-800 px-1">
							<WatchLaterBtn video={vid.data.summary} />
						</span>
						<button
							type="button"
							onClick={() => setPickerOpen(true)}
							className="flex items-center gap-1.5 rounded-full bg-[#e62117] px-4 py-2 text-sm font-semibold text-white hover:bg-[#f03428] transition-colors"
						>
							<Download size={15} />
							Download
						</button>
					</div>
				</div>
				<FormatPicker
					videoId={v}
					videoTitle={vid.data.summary.title}
					open={pickerOpen}
					onClose={() => setPickerOpen(false)}
				/>
			</div>
			<aside>
				<h2 className="font-semibold mb-2">Related</h2>
				<div className="flex flex-col gap-3">
					{rel.data?.map((r) => (
						<VideoCard key={r.id} v={r} />
					))}
				</div>
			</aside>
		</div>
	);
}
