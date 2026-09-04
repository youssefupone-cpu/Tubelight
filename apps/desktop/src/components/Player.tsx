import { MediaPlayer, MediaProvider } from "@vidstack/react";
import {
	DefaultVideoLayout,
	defaultLayoutIcons,
} from "@vidstack/react/player/layouts/default";
import "@vidstack/react/player/styles/default/theme.css";
import "@vidstack/react/player/styles/default/layouts/video.css";

export function Player({ src, poster }: { src: string; poster?: string }) {
	return (
		<MediaPlayer
			src={src}
			poster={poster}
			title="yoube"
			crossOrigin="anonymous"
		>
			<MediaProvider />
			<DefaultVideoLayout icons={defaultLayoutIcons} />
		</MediaPlayer>
	);
}
