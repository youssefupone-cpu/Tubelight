import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { download_enqueue, list_formats } from "@yoube/contracts";
import { Download, X } from "lucide-react";
import { useState } from "react";
import { cn } from "../lib/utils";

function defaultDestDir(): string {
	try {
		return (
			localStorage.getItem("yoube.download_dir") ||
			(navigator.platform.includes("Win") ? "Downloads" : "/tmp")
		);
	} catch {
		return "/tmp";
	}
}

export function FormatPicker({
	videoId,
	videoTitle,
	open,
	onClose,
}: {
	videoId: string;
	videoTitle: string;
	open: boolean;
	onClose: () => void;
}) {
	const qc = useQueryClient();
	const [destDir, setDestDir] = useState(defaultDestDir);
	const formats = useQuery({
		queryKey: ["formats", videoId],
		queryFn: () => list_formats(videoId),
		enabled: open && !!videoId,
	});
	const enqueue = useMutation({
		mutationFn: (format_id: string) => {
			try {
				localStorage.setItem("yoube.download_dir", destDir);
			} catch {
				/* private mode — ignore */
			}
			return download_enqueue(videoId, format_id, destDir);
		},
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ["downloads"] });
			onClose();
		},
	});

	if (!open) return null;
	return (
		// biome-ignore lint/a11y/useKeyWithClickEvents: backdrop click-to-close is a deliberate modal pattern
		<dialog
			open
			className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			onClick={onClose}
			aria-label={`Download ${videoTitle}`}
		>
			{/* biome-ignore lint/a11y/useKeyWithClickEvents: stopPropagation container, not interactive */}
			<div
				className="w-full max-w-lg rounded-xl border border-neutral-800 bg-neutral-900 p-4"
				onClick={(e) => e.stopPropagation()}
			>
				<div className="flex items-center justify-between">
					<h2 className="font-semibold">Download</h2>
					<button
						type="button"
						onClick={onClose}
						className="p-1 rounded hover:bg-neutral-800 text-neutral-400"
						title="Close"
					>
						<X size={18} />
					</button>
				</div>
				<p className="text-sm text-neutral-400 mt-1 line-clamp-2">
					{videoTitle}
				</p>
				<label className="block mt-3 text-sm">
					<span className="text-neutral-400">Destination folder</span>
					<input
						value={destDir}
						onChange={(e) => setDestDir(e.target.value)}
						className="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-800 px-3 py-2 text-sm outline-none focus:border-neutral-500"
						placeholder="/tmp"
					/>
				</label>
				<div className="mt-3 max-h-72 overflow-auto rounded-lg border border-neutral-800">
					{formats.isPending && (
						<div className="p-4 text-sm text-neutral-400">Loading formats…</div>
					)}
					{formats.error && (
						<div className="p-4 text-sm text-red-400">
							{String(formats.error)}
						</div>
					)}
					{formats.data?.length === 0 && !formats.isPending && (
						<div className="p-4 text-sm text-neutral-400">
							No formats available for this video.
						</div>
					)}
					{formats.data?.map((f) => (
						<button
							type="button"
							key={f.format_id}
							disabled={enqueue.isPending}
							onClick={() => enqueue.mutate(f.format_id)}
							className={cn(
								"flex w-full items-center justify-between gap-3 px-3 py-2 text-left text-sm hover:bg-neutral-800 disabled:opacity-50",
								"border-b border-neutral-800 last:border-0",
							)}
						>
							<span className="font-mono text-xs text-neutral-300">
								{f.format_id} · {f.ext} · {f.resolution ?? f.note ?? "audio"}
							</span>
							<span className="flex items-center gap-1 text-neutral-400">
								<Download size={14} />
							</span>
						</button>
					))}
				</div>
				{enqueue.error && (
					<div className="mt-2 text-sm text-red-400">
						{String(enqueue.error)}
					</div>
				)}
			</div>
		</dialog>
	);
}
