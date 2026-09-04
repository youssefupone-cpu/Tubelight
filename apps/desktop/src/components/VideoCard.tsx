import { Link } from "@tanstack/react-router";
import type { VideoSummary } from "@yoube/contracts";

export function VideoCard({ v }: { v: VideoSummary }) {
	return (
		<Link
			to="/watch"
			search={{ v: v.id }}
			className="block group"
		>
			<div className="aspect-video bg-neutral-800 rounded-xl overflow-hidden">
				{v.thumbnail_url && (
					<img
						src={v.thumbnail_url}
						alt=""
						className="w-full h-full object-cover group-hover:scale-[1.02] transition-transform"
					/>
				)}
			</div>
			<div className="mt-2">
				<div className="font-medium line-clamp-2">{v.title}</div>
				<div className="text-sm text-neutral-400">{v.channel_title}</div>
			</div>
		</Link>
	);
}
