import { APP_NAME, APP_VERSION, GITHUB_REPO } from "./appmeta";

export interface BugReport {
	title: string;
	error?: string;
	route?: string;
	extra?: string;
}

/**
 * Build a prefilled GitHub issue URL (opens the "new issue" form with
 * template fields filled). No network call, no tracking — the user reviews
 * everything before submitting. Labels the issue as `bug` for the 48h
 * triage SLA (see SECURITY.md).
 */
export function buildIssueUrl(report: BugReport): string {
	const body = [
		"### What happened",
		"",
		report.error
			? `\`\`\`\n${report.error}\n\`\`\``
			: "_Describe the problem here._",
		"",
		"### App",
		"",
		`- ${APP_NAME} v${APP_VERSION}`,
		`- Platform: ${platformLabel()}`,
		report.route ? `- Route: ${report.route}` : null,
		report.extra ? `- Notes: ${report.extra}` : null,
		"",
		"_Reported from the in-app bug reporter. Logs/user data are never attached automatically._",
	]
		.filter((l) => l !== null)
		.join("\n");
	const params = new URLSearchParams({
		title: report.title || `[bug] ${APP_NAME} v${APP_VERSION}`,
		body,
		labels: "bug",
	});
	return `https://github.com/${GITHUB_REPO}/issues/new?${params.toString()}`;
}

/** One-line plain-text report for clipboard fallback. */
export function buildReportText(report: BugReport): string {
	return [
		`${APP_NAME} v${APP_VERSION} — ${report.title || "bug report"}`,
		`Platform: ${platformLabel()}`,
		report.route ? `Route: ${report.route}` : null,
		report.error ? `Error: ${report.error}` : null,
		report.extra ? `Notes: ${report.extra}` : null,
	]
		.filter((l) => l !== null)
		.join("\n");
}

function platformLabel(): string {
	if (typeof navigator === "undefined") return "unknown";
	const ua = navigator.userAgent ?? "";
	const os = /Windows/.test(ua)
		? "Windows"
		: /Mac OS/.test(ua)
			? "macOS"
			: /Android/.test(ua)
				? "Android"
				: /Linux/.test(ua)
					? "Linux"
					: "unknown OS";
	const inTauri =
		typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
	return `${os}${inTauri ? " (Tauri app)" : " (browser preview)"}`;
}
