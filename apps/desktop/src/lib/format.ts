// Display formatters for video metadata (durations, view counts, dates).

/** 735 -> "12:15", 3661 -> "1:01:01", null -> "" */
export function formatDuration(totalSeconds?: number | null): string {
	if (totalSeconds == null || totalSeconds < 0) return "";
	const s = Math.floor(totalSeconds);
	const h = Math.floor(s / 3600);
	const m = Math.floor((s % 3600) / 60);
	const sec = s % 60;
	const mm = h > 0 ? String(m).padStart(2, "0") : String(m);
	return `${h > 0 ? `${h}:` : ""}${mm}:${String(sec).padStart(2, "0")}`;
}

/** 2400000 -> "2.4M views", 980000 -> "980K views", null -> "" */
export function formatViews(views?: number | null): string {
	if (views == null) return "";
	if (views >= 1_000_000_000) return `${trimNum(views / 1_000_000_000)}B views`;
	if (views >= 1_000_000) return `${trimNum(views / 1_000_000)}M views`;
	if (views >= 1_000) return `${trimNum(views / 1_000)}K views`;
	return `${views} views`;
}

function trimNum(n: number): string {
	const r = Math.round(n * 10) / 10;
	return Number.isInteger(r) ? String(r) : r.toFixed(1);
}

/**
 * "20260312" (yt-dlp upload_date) -> "6 months ago".
 * Falls back to "" for missing/garbled input (fail-open for mocks).
 */
export function timeAgo(yyyymmdd?: string | null): string {
	if (!yyyymmdd || yyyymmdd.length !== 8) return "";
	const y = Number(yyyymmdd.slice(0, 4));
	const m = Number(yyyymmdd.slice(4, 6)) - 1;
	const d = Number(yyyymmdd.slice(6, 8));
	const then = new Date(y, m, d).getTime();
	if (Number.isNaN(then)) return "";
	const days = Math.max(0, Math.floor((Date.now() - then) / 86_400_000));
	if (days < 1) return "today";
	if (days < 7) return `${days} day${days === 1 ? "" : "s"} ago`;
	if (days < 30) {
		const w = Math.floor(days / 7);
		return `${w} week${w === 1 ? "" : "s"} ago`;
	}
	if (days < 365) {
		const mo = Math.floor(days / 30);
		return `${mo} month${mo === 1 ? "" : "s"} ago`;
	}
	const yr = Math.floor(days / 365);
	return `${yr} year${yr === 1 ? "" : "s"} ago`;
}

/** "Fireship" -> "F", "الدحيح" -> "د", "" -> "?" */
export function initialOf(name?: string | null): string {
	const t = (name ?? "").trim();
	if (!t) return "?";
	return [...t][0].toUpperCase();
}
