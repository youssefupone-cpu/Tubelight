import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import { get_video, related } from "@yoube/contracts";
import { Player } from "../components/Player";
import { VideoCard } from "../components/VideoCard";

export const Route = createFileRoute("/watch")({
	validateSearch: (search) => ({
		v: (search as { v?: string }).v ?? "",
	}),
	component: Watch,
});

function Watch() {
	const { v } = useSearch({ from: "/watch" });
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
	if (vid.isPending) return <div className="p-4">Loading…</div>;
	if (vid.error) return <div className="p-4 text-red-400">{String(vid.error)}</div>;
	const best =
		vid.data.formats.find(
			(f) =>
				(f.vcodec ?? "none") !== "none" &&
				(f.acodec ?? "none") !== "none",
		) ?? vid.data.formats[0];
	return (
		<div className="h-full overflow-auto p-4 grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-6">
			<div>
				<div className="aspect-video bg-black rounded-xl overflow-hidden">
					<Player
						src={best.url ?? ""}
						poster={vid.data.summary.thumbnail_url ?? undefined}
					/>
				</div>
				<h1 className="text-xl font-semibold mt-3">{vid.data.summary.title}</h1>
				<div className="text-sm text-neutral-400">
					{vid.data.summary.channel_title}
				</div>
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
