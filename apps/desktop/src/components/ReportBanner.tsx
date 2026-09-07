import { AlertTriangle, Check, Copy, ExternalLink, X } from "lucide-react";
import { useState } from "react";
import { clearError, useCapturedError } from "../lib/errorbus";
import { buildIssueUrl, buildReportText } from "../lib/report";

/**
 * Global crash banner: appears when window.onerror / unhandledrejection
 * fires. Copy-first flow works everywhere (browser + Tauri webview);
 * "Open issue" is a plain anchor the user reviews before submitting.
 * Nothing is ever sent automatically.
 */
export function ReportBanner() {
	const err = useCapturedError();
	const [copied, setCopied] = useState(false);
	if (!err) return null;

	const report = {
		title: "[bug] Uncaught error",
		error: `${err.message} (at ${err.at})`,
		route: err.route,
	};
	const url = buildIssueUrl(report);

	const copy = async () => {
		const text = buildReportText(report);
		try {
			await navigator.clipboard.writeText(text);
		} catch {
			const ta = document.createElement("textarea");
			ta.value = text;
			document.body.appendChild(ta);
			ta.select();
			document.execCommand("copy");
			ta.remove();
		}
		setCopied(true);
	};

	return (
		<div
			role="alert"
			className="flex items-start gap-3 border-b border-red-900/60 bg-red-950/60 px-4 py-2.5"
		>
			<AlertTriangle size={17} className="mt-0.5 flex-shrink-0 text-red-400" />
			<div className="min-w-0 flex-1">
				<div className="text-sm font-medium text-red-200">
					Something went wrong
				</div>
				<div className="truncate text-xs text-red-300/80">{err.message}</div>
			</div>
			<button
				type="button"
				onClick={copy}
				className="flex items-center gap-1.5 rounded-full bg-neutral-800 px-3 py-1.5 text-xs font-medium text-neutral-200 hover:bg-neutral-700"
			>
				{copied ? <Check size={13} /> : <Copy size={13} />}
				{copied ? "Copied" : "Copy report"}
			</button>
			<a
				href={url}
				target="_blank"
				rel="noopener noreferrer"
				className="flex items-center gap-1.5 rounded-full bg-neutral-800 px-3 py-1.5 text-xs font-medium text-neutral-200 hover:bg-neutral-700"
			>
				<ExternalLink size={13} />
				Open issue
			</a>
			<button
				type="button"
				onClick={() => {
					clearError();
					setCopied(false);
				}}
				aria-label="Dismiss"
				className="rounded-full p-1.5 text-neutral-400 hover:bg-neutral-800 hover:text-white"
			>
				<X size={15} />
			</button>
		</div>
	);
}
