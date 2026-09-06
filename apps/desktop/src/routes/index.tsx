import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Region, trending } from "@yoube/contracts";
import { VideoGridSkeleton } from "../components/VideoCard";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
	const q = useQuery({
		queryKey: ["trending", Region.US],
		queryFn: () => trending(Region.US),
	});
	if (q.isPending) return <VideoGridSkeleton count={8} />;
	if (q.error) return <div className="p-4 text-red-400">{String(q.error)}</div>;
	return (
		<div className="h-full overflow-auto">
			<div className="flex items-baseline gap-3 px-4 pt-4">
				<h1 className="text-lg font-bold">Trending</h1>
				<span className="text-xs text-neutral-500">
					United States · updated hourly
				</span>
			</div>
			<VideoGrid items={q.data} />
		</div>
	);
}
