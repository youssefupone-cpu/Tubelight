import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { VideoCard } from "./VideoCard";
import type { VideoSummary } from "@yoube/contracts";

export function VideoGrid({ items }: { items: VideoSummary[] }) {
	const parent = useRef<HTMLDivElement>(null);
	const row = useVirtualizer({
		count: items.length,
		getScrollElement: () => parent.current,
		estimateSize: () => 240,
		overscan: 6,
	});
	return (
		<div ref={parent} className="h-full overflow-auto p-4">
			<div style={{ height: row.getTotalSize(), position: "relative" }}>
				{row.getVirtualItems().map((v) => (
					<div
						key={v.key}
						style={{
							position: "absolute",
							top: v.start,
							left: 0,
							right: 0,
						}}
						className="px-1"
					>
						<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
							{items
								.slice(v.index, v.index + 4)
								.map((it) => (
									<VideoCard key={it.id} v={it} />
								))}
						</div>
					</div>
				))}
			</div>
		</div>
	);
}
