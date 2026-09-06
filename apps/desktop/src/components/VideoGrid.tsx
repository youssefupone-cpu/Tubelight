import type { VideoSummary } from "@yoube/contracts";
import { VideoCard } from "./VideoCard";

/**
 * Plain responsive grid.
 *
 * NOTE: this deliberately replaces the earlier TanStack-Virtual prototype,
 * which treated every *item* as a virtual *row* while rendering 4 items per
 * row — rows overlapped and videos repeated on screen. A plain grid is
 * correct for feed-sized lists; reintroduce virtualization only with a
 * row-based model (chunk items into rows first, measure real row heights).
 */
export function VideoGrid({ items }: { items: VideoSummary[] }) {
	return (
		<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-x-4 gap-y-6 p-4">
			{items.map((it) => (
				<VideoCard key={it.id} v={it} />
			))}
		</div>
	);
}
