import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { trending, Region } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
	const q = useQuery({
		queryKey: ["trending", Region.US],
		queryFn: () => trending(Region.US),
	});
	if (q.isPending) return <div className="p-4">Loading…</div>;
	if (q.error) return <div className="p-4 text-red-400">{String(q.error)}</div>;
	return <VideoGrid items={q.data} />;
}
