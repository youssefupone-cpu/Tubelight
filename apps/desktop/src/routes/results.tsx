import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import { search } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/results")({
  validateSearch: (search) => ({ q: (search as { q?: string }).q ?? "" }),
  component: Results,
});
function Results() {
  const { q } = useSearch({ from: "/results" });
  const res = useQuery({ queryKey: ["search", q], queryFn: () => search(q, 0), enabled: !!q });
  if (res.isPending) return <div className="p-4">Searching…</div>;
  return <VideoGrid items={res.data ?? []} />;
}
