import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
	type Job,
	download_cancel,
	download_list,
	download_pause,
	download_resume,
} from "@yoube/contracts";
import { Pause, Play, X } from "lucide-react";
import { useState } from "react";
import { cn } from "../lib/utils";

export const Route = createFileRoute("/downloads")({
	component: Downloads,
});

type Tab = "active" | "queued" | "completed" | "failed";

function tabOf(j: Job): Tab {
	switch (j.state) {
		case "Running":
		case "Paused":
			return "active";
		case "Queued":
			return "queued";
		case "Done":
			return "completed";
		default:
			return "failed";
	}
}

function pct(j: Job): number | null {
	if (j.total_bytes == null || j.total_bytes === 0) return null;
	return Math.min(100, Math.round((j.progress_bytes / j.total_bytes) * 100));
}

function JobRow({ job }: { job: Job }) {
	const qc = useQueryClient();
	const invalidate = () => qc.invalidateQueries({ queryKey: ["downloads"] });
	const pause = useMutation({
		mutationFn: () => download_pause(job.id),
		onSuccess: invalidate,
	});
	const resume = useMutation({
		mutationFn: () => download_resume(job.id),
		onSuccess: invalidate,
	});
	const cancel = useMutation({
		mutationFn: () => download_cancel(job.id),
		onSuccess: invalidate,
	});
	const p = pct(job);
	return (
		<div className="rounded-xl border border-neutral-800 bg-neutral-900 p-3">
			<div className="flex items-center justify-between gap-3">
				<div className="min-w-0">
					<div className="truncate font-mono text-sm">{job.video_id}</div>
					<div className="text-xs text-neutral-400">
						{job.format_id} · {job.state}
						{job.eta_s != null && ` · ETA ${job.eta_s}s`}
						{job.error && ` · ${job.error}`}
					</div>
				</div>
				<div className="flex items-center gap-1">
					{job.state === "Running" && (
						<button
							type="button"
							onClick={() => pause.mutate()}
							className="p-1.5 rounded hover:bg-neutral-800 text-neutral-300"
							title="Pause"
						>
							<Pause size={16} />
						</button>
					)}
					{job.state === "Paused" && (
						<button
							type="button"
							onClick={() => resume.mutate()}
							className="p-1.5 rounded hover:bg-neutral-800 text-neutral-300"
							title="Resume"
						>
							<Play size={16} />
						</button>
					)}
					{(job.state === "Running" ||
						job.state === "Paused" ||
						job.state === "Queued") && (
						<button
							type="button"
							onClick={() => cancel.mutate()}
							className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400"
							title="Cancel"
						>
							<X size={16} />
						</button>
					)}
				</div>
			</div>
			<div className="mt-2 h-1.5 overflow-hidden rounded-full bg-neutral-800">
				<div
					className={cn(
						"h-full rounded-full transition-all",
						job.state === "Failed" || job.state === "Cancelled"
							? "bg-red-500"
							: "bg-white",
					)}
					style={{ width: `${p ?? (job.state === "Done" ? 100 : 4)}%` }}
				/>
			</div>
		</div>
	);
}

function Downloads() {
	const [tab, setTab] = useState<Tab>("active");
	const jobs = useQuery({
		queryKey: ["downloads"],
		queryFn: download_list,
		// Poll the Rust queue: cheap snapshot, no event plumbing needed.
		refetchInterval: 1000,
	});
	const items = (jobs.data ?? []).filter((j) => tabOf(j) === tab);
	const counts: Record<Tab, number> = {
		active: 0,
		queued: 0,
		completed: 0,
		failed: 0,
	};
	for (const j of jobs.data ?? []) counts[tabOf(j)] += 1;

	return (
		<div className="h-full overflow-auto p-4">
			<h1 className="text-xl font-semibold">Downloads</h1>
			<div className="mt-3 flex gap-2">
				{(["active", "queued", "completed", "failed"] as Tab[]).map((t) => (
					<button
						type="button"
						key={t}
						onClick={() => setTab(t)}
						className={cn(
							"rounded-full px-3 py-1 text-sm capitalize",
							tab === t
								? "bg-white text-black"
								: "bg-neutral-800 text-neutral-300 hover:bg-neutral-700",
						)}
					>
						{t} ({counts[t]})
					</button>
				))}
			</div>
			{jobs.isPending && (
				<div className="mt-4 text-sm text-neutral-400">Loading…</div>
			)}
			{jobs.error && (
				<div className="mt-4 text-sm text-red-400">{String(jobs.error)}</div>
			)}
			<div className="mt-4 flex flex-col gap-3">
				{items.map((j) => (
					<JobRow key={j.id} job={j} />
				))}
				{!jobs.isPending && items.length === 0 && (
					<div className="text-sm text-neutral-500">
						Nothing here. Pick a format from any video to start a download.
					</div>
				)}
			</div>
		</div>
	);
}
