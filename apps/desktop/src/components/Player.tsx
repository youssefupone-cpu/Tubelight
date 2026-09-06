import { MediaPlayer, MediaProvider } from "@vidstack/react";
import {
	DefaultVideoLayout,
	defaultLayoutIcons,
} from "@vidstack/react/player/layouts/default";
import { useRef } from "react";
import "@vidstack/react/player/styles/default/theme.css";
import "@vidstack/react/player/styles/default/layouts/video.css";
import type { Segment } from "@yoube/contracts";
import { useSponsorSkip } from "../hooks/useSponsorSkip";

export function Player({
	src,
	poster,
	segments = [],
	skipEnabled = false,
	skipCategories = [],
}: {
	src: string;
	poster?: string;
	segments?: Segment[];
	skipEnabled?: boolean;
	skipCategories?: readonly string[];
}) {
	const container = useRef<HTMLDivElement | null>(null);
	useSponsorSkip(container, segments, skipCategories, skipEnabled);
	return (
		<div ref={container}>
			<MediaPlayer
				src={src}
				poster={poster}
				title="yoube"
				crossOrigin="anonymous"
			>
				<MediaProvider />
				<DefaultVideoLayout icons={defaultLayoutIcons} />
			</MediaPlayer>
		</div>
	);
}
