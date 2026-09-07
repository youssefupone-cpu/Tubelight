import { convertFileSrc } from "@tauri-apps/api/core";

export function isTauri(): boolean {
	return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Resolve a media URL for the current runtime. Plain https/blob/data URLs
 * pass through untouched. Local absolute paths (e.g. `/mock/sample.mp4` in
 * previews, or downloaded files in production) must go through Tauri's
 * asset protocol inside the desktop app — a raw `/...` path never loads in
 * the webview. Safe to call in a plain browser (returns input as-is).
 */
export function resolveMediaSrc(src: string): string {
	if (!src) return src;
	if (/^(https?|blob|data|asset):/.test(src)) return src;
	if (isTauri()) {
		try {
			return convertFileSrc(src);
		} catch {
			return src;
		}
	}
	return src;
}
