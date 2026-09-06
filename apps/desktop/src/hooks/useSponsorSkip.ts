import type { Segment } from "@yoube/contracts";
import { type RefObject, useEffect } from "react";

/** Auto-skip SponsorBlock segments during playback (spec §7 L3).
 *
 * Polls the underlying `<video>` element every 250ms; when `currentTime`
 * falls inside a segment whose category is enabled, seeks just past its end.
 * Operates on the DOM node (not the vidstack API) so it survives player
 * upgrades. Fail-open: any error stops the interval silently.
 */
export function useSponsorSkip(
	container: RefObject<HTMLElement | null>,
	segments: Segment[],
	enabledCategories: readonly string[],
	enabled: boolean,
) {
	useEffect(() => {
		if (!enabled || segments.length === 0 || enabledCategories.length === 0) {
			return;
		}
		const id = window.setInterval(() => {
			try {
				const video = container.current?.querySelector("video");
				if (!video || video.seeking || video.paused) return;
				const t = video.currentTime;
				const hit = segments.find(
					(s) =>
						enabledCategories.includes(s.category) &&
						t >= s.start_s &&
						t < s.end_s,
				);
				if (hit && Number.isFinite(hit.end_s)) {
					video.currentTime = hit.end_s + 0.05;
				}
			} catch {
				/* fail-open: leave playback alone */
			}
		}, 250);
		return () => window.clearInterval(id);
	}, [container, segments, enabledCategories, enabled]);
}
