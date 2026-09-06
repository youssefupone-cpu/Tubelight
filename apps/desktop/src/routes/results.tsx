import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import { search } from "@yoube/contracts";
import { SearchX } from "lucide-react";
import { EmptyState } from "../components/EmptyState";
import { VideoGridSkeleton } from "../components/VideoCard";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/results")({
	validateSearch: (search) => ({ q: (search as { q?: string }).q ?? "" }),
	component: Results,
});
function Results() {
	const { q } = useSearch({ from: "/results" });
	const res = useQuery({
		queryKey: ["search", q],
		queryFn: () => search(q, 0),
		enabled: !!q,
	});
	if (!q)
		return (
			<EmptyState
				icon={SearchX}
				title="Search for something"
				hint="Try a video title, topic, or channel name."
			/>
		);
	if (res.isPending) return <VideoGridSkeleton count={8} />;
	if (res.error)
		return <div className="p-4 text-red-400">{String(res.error)}</div>;
	if ((res.data ?? []).length === 0)
		return (
			<EmptyState
				icon={SearchX}
				title={`No results for "${q}"`}
				hint="Check the spelling or try different keywords."
			/>
		);
	return (
		<div className="h-full overflow-auto">
			<div className="px-4 pt-4">
				<h1 className="text-lg font-bold" dir="auto">
					Results for “{q}”
				</h1>
			</div>
			<VideoGrid items={res.data ?? []} />
		</div>
	);
}
