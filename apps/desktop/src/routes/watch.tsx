import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import { filter_segments_for, get_video, related } from "@yoube/contracts";
import { useState } from "react";
import { FormatPicker } from "../components/FormatPicker";
import { Player } from "../components/Player";
import { VideoCard } from "../components/VideoCard";
import { DEFAULT_SKIP_CATEGORIES, useSetting } from "../hooks/useSettings";

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
				<div className="aspect-video bg-black rounded-xl overflow-hidden">
					<Player
						src={best.url ?? ""}
						poster={vid.data.summary.thumbnail_url ?? undefined}
						segments={segments.data ?? []}
						skipEnabled={l3Enabled}
						skipCategories={skipCats}
					/>
				</div>
				<h1 className="text-xl font-semibold mt-3">{vid.data.summary.title}</h1>
				<div className="text-sm text-neutral-400">
					{vid.data.summary.channel_title}
					{(segments.data?.length ?? 0) > 0 && (
						<span className="ml-2 rounded-full bg-neutral-800 px-2 py-0.5 text-xs">
							{segments.data?.length} skips
						</span>
					)}
				</div>
				<div className="mt-2">
					<button
						type="button"
						onClick={() => setPickerOpen(true)}
						className="rounded-full bg-white px-4 py-1.5 text-sm font-medium text-black hover:bg-neutral-200"
					>
						Download
					</button>
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
