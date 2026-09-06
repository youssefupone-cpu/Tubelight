import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, createFileRoute } from "@tanstack/react-router";
import {
	dns_install,
	dns_refresh,
	dns_status,
	dns_uninstall,
	filter_init,
} from "@yoube/contracts";
import { DEFAULT_SKIP_CATEGORIES, useSetting } from "../hooks/useSettings";
import { cn } from "../lib/utils";

export const Route = createFileRoute("/settings")({
	validateSearch: (search) => ({
		tab: (search as { tab?: string }).tab ?? "general",
	}),
	component: Settings,
});

type Tab = "general" | "player" | "downloads" | "privacy" | "account" | "about";
const TABS: Tab[] = [
	"general",
	"player",
	"downloads",
	"privacy",
	"account",
	"about",
];

const ALL_CATEGORIES = [
	"sponsor",
	"intro",
	"outro",
	"selfpromo",
	"preview",
	"music_offtopic",
	"filler",
	"interaction",
	"poi_highlight",
] as const;

function Toggle({
	label,
	hint,
	checked,
	onChange,
}: {
	label: string;
	hint?: string;
	checked: boolean;
	onChange: (v: boolean) => void;
}) {
	return (
		<label className="flex items-start justify-between gap-4 py-2">
			<span>
				<span className="block text-sm font-medium">{label}</span>
				{hint && <span className="block text-xs text-neutral-400">{hint}</span>}
			</span>
			<input
				type="checkbox"
				checked={checked}
				onChange={(e) => onChange(e.target.checked)}
				className="mt-1 h-4 w-4 accent-white"
			/>
		</label>
	);
}

function Privacy() {
	const qc = useQueryClient();
	const [l2, setL2] = useSetting("l2_enabled", true);
	const [l3, setL3] = useSetting("l3_enabled", true);
	const [cats, setCats] = useSetting<readonly string[]>(
		"l3_categories",
		DEFAULT_SKIP_CATEGORIES,
	);
	const status = useQuery({
		queryKey: ["dns-status"],
		queryFn: dns_status,
	});
	const invalidateDns = () =>
		qc.invalidateQueries({ queryKey: ["dns-status"] });
	const install = useMutation({
		mutationFn: dns_install,
		onSuccess: invalidateDns,
	});
	const uninstall = useMutation({
		mutationFn: dns_uninstall,
		onSuccess: invalidateDns,
	});
	const refresh = useMutation({
		mutationFn: dns_refresh,
		onSuccess: invalidateDns,
	});
	const initLists = useMutation({ mutationFn: filter_init });
	const installed = status.data?.startsWith("installed") ?? false;

	const toggleCat = (c: string) => {
		setCats(cats.includes(c) ? cats.filter((x) => x !== c) : [...cats, c]);
	};

	return (
		<div className="flex flex-col gap-6">
			<section className="rounded-xl border border-neutral-800 bg-neutral-900 p-4">
				<h2 className="font-semibold">L1 — System hosts (StevenBlack)</h2>
				<p className="mt-1 text-xs text-neutral-400">
					Status: {status.data ?? "…"} · requires admin/root to change.
				</p>
				{install.error && (
					<p className="mt-2 text-xs text-red-400">{String(install.error)}</p>
				)}
				<div className="mt-3 flex flex-wrap gap-2">
					<button
						type="button"
						onClick={() => install.mutate()}
						disabled={install.isPending || installed}
						className="rounded-full bg-white px-4 py-1.5 text-sm font-medium text-black hover:bg-neutral-200 disabled:opacity-50"
					>
						Install block
					</button>
					<button
						type="button"
						onClick={() => refresh.mutate()}
						disabled={refresh.isPending || !installed}
						className="rounded-full bg-neutral-800 px-4 py-1.5 text-sm text-neutral-200 hover:bg-neutral-700 disabled:opacity-50"
					>
						Refresh lists
					</button>
					<button
						type="button"
						onClick={() => uninstall.mutate()}
						disabled={uninstall.isPending || !installed}
						className="rounded-full bg-neutral-800 px-4 py-1.5 text-sm text-red-300 hover:bg-neutral-700 disabled:opacity-50"
					>
						Uninstall (restore)
					</button>
				</div>
			</section>

			<section className="rounded-xl border border-neutral-800 bg-neutral-900 p-4">
				<h2 className="font-semibold">L2 — In-app ABP filter</h2>
				<Toggle
					label="Enable network filtering"
					hint="Every Rust-side request is screened (EasyList + EasyPrivacy + YouTube rules)."
					checked={l2}
					onChange={setL2}
				/>
				<div className="mt-2 flex items-center gap-2">
					<button
						type="button"
						onClick={() => initLists.mutate()}
						disabled={initLists.isPending}
						className="rounded-full bg-neutral-800 px-4 py-1.5 text-sm text-neutral-200 hover:bg-neutral-700 disabled:opacity-50"
					>
						{initLists.isPending ? "Fetching…" : "Fetch latest lists"}
					</button>
					{initLists.data != null && (
						<span className="text-xs text-neutral-400">
							{initLists.data} rules loaded
						</span>
					)}
				</div>
				{initLists.error && (
					<p className="mt-2 text-xs text-red-400">{String(initLists.error)}</p>
				)}
			</section>

			<section className="rounded-xl border border-neutral-800 bg-neutral-900 p-4">
				<h2 className="font-semibold">L3 — Player segment skip</h2>
				<Toggle
					label="Auto-skip sponsored segments"
					hint="Uses SponsorBlock data, cached 24h. Fail-open: playback never breaks."
					checked={l3}
					onChange={setL3}
				/>
				<div className="mt-2 grid grid-cols-2 gap-1 sm:grid-cols-3">
					{ALL_CATEGORIES.map((c) => (
						<label
							key={c}
							className="flex items-center gap-2 text-sm text-neutral-300"
						>
							<input
								type="checkbox"
								checked={cats.includes(c)}
								onChange={() => toggleCat(c)}
								className="h-4 w-4 accent-white"
							/>
							{c}
						</label>
					))}
				</div>
				<p className="mt-3 text-xs text-neutral-500">
					Missing a segment? Use the SponsorBlock web submitter to report it.
				</p>
			</section>
		</div>
	);
}

function Settings() {
	const { tab } = Route.useSearch();
	const [region, setRegion] = useSetting("default_region", "US");
	const [downloadDir, setDownloadDir] = useSetting("download_dir", "/tmp");
	const [l3, setL3] = useSetting("l3_enabled", true);
	const [cats] = useSetting<readonly string[]>(
		"l3_categories",
		DEFAULT_SKIP_CATEGORIES,
	);

	return (
		<div className="h-full overflow-auto p-4">
			<h1 className="text-xl font-semibold">Settings</h1>
			<div className="mt-3 flex flex-wrap gap-2">
				{TABS.map((t) => (
					<Link
						key={t}
						to="/settings"
						search={{ tab: t }}
						className={cn(
							"rounded-full px-3 py-1 text-sm capitalize",
							tab === t
								? "bg-white text-black"
								: "bg-neutral-800 text-neutral-300 hover:bg-neutral-700",
						)}
					>
						{t}
					</Link>
				))}
			</div>

			<div className="mt-4 max-w-2xl">
				{tab === "general" && (
					<div className="rounded-xl border border-neutral-800 bg-neutral-900 p-4">
						<label className="block text-sm">
							<span className="text-neutral-400">Default region</span>
							<input
								value={region}
								onChange={(e) => setRegion(e.target.value.toUpperCase())}
								maxLength={2}
								className="mt-1 w-24 rounded-lg border border-neutral-700 bg-neutral-800 px-3 py-2 text-sm outline-none focus:border-neutral-500"
							/>
						</label>
					</div>
				)}
				{tab === "player" && (
					<div className="rounded-xl border border-neutral-800 bg-neutral-900 p-4">
						<Toggle
							label="Sponsor skip"
							hint="Same master switch as Privacy → L3."
							checked={l3}
							onChange={setL3}
						/>
						<p className="mt-2 text-xs text-neutral-400">
							Categories: {cats.join(", ") || "none"}
						</p>
					</div>
				)}
				{tab === "downloads" && (
					<div className="rounded-xl border border-neutral-800 bg-neutral-900 p-4">
						<label className="block text-sm">
							<span className="text-neutral-400">Default download folder</span>
							<input
								value={downloadDir}
								onChange={(e) => setDownloadDir(e.target.value)}
								className="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-800 px-3 py-2 text-sm outline-none focus:border-neutral-500"
							/>
						</label>
						<p className="mt-2 text-xs text-neutral-500">
							Up to 2 concurrent downloads. Partial `.part` files resume on
							retry.
						</p>
					</div>
				)}
				{tab === "privacy" && <Privacy />}
				{tab === "account" && (
					<div className="rounded-xl border border-neutral-800 bg-neutral-900 p-4 text-sm text-neutral-300">
						Manage virtual users, export and import from the{" "}
						<Link to="/users" className="underline">
							Users
						</Link>{" "}
						page.
					</div>
				)}
				{tab === "about" && (
					<div className="rounded-xl border border-neutral-800 bg-neutral-900 p-4 text-sm text-neutral-300">
						<p className="font-semibold text-white">yoube 0.0.1</p>
						<p className="mt-1 text-xs text-neutral-400">
							Tauri 2 + React 19 desktop client for YouTube. Sidecars: yt-dlp +
							ffmpeg. Block lists: EasyList, EasyPrivacy, StevenBlack hosts.
							Segments: SponsorBlock + DeArrow.
						</p>
					</div>
				)}
			</div>
		</div>
	);
}
