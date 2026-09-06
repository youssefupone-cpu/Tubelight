import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
	create_user,
	current_user,
	delete_user,
	export_account,
	import_account,
	switch_user,
} from "@yoube/contracts";
import { useState } from "react";

export const Route = createFileRoute("/users")({
	component: Users,
});

function Users() {
	const qc = useQueryClient();
	const user = useQuery({
		queryKey: ["current-user"],
		queryFn: current_user,
	});

	const [newName, setNewName] = useState("");
	const [switchId, setSwitchId] = useState("1");
	const [exportPath, setExportPath] = useState("~/yoube-export.json");
	const [importPath, setImportPath] = useState("~/yoube-export.json");
	const [exportMsg, setExportMsg] = useState<string | null>(null);
	const [importMsg, setImportMsg] = useState<string | null>(null);

	const createMut = useMutation({
		mutationFn: (name: string) => create_user(name),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ["current-user"] });
			setNewName("");
		},
	});

	const switchMut = useMutation({
		mutationFn: (id: number) => switch_user(id),
		onSuccess: () => qc.invalidateQueries({ queryKey: ["current-user"] }),
	});

	const deleteMut = useMutation({
		mutationFn: (id: number) => delete_user(id),
		onSuccess: () => qc.invalidateQueries({ queryKey: ["current-user"] }),
	});

	const exportMut = useMutation({
		mutationFn: (dest: string) => export_account(dest),
		onSuccess: (resolved) => {
			setExportMsg(`Saved to ${resolved}`);
			qc.invalidateQueries({ queryKey: ["current-user"] });
		},
		onError: (e) => setExportMsg(`Export failed: ${String(e)}`),
	});

	const importMut = useMutation({
		mutationFn: (src: string) => import_account(src),
		onSuccess: () => {
			setImportMsg("Import complete");
			qc.invalidateQueries({ queryKey: ["current-user"] });
			qc.invalidateQueries({ queryKey: ["subscriptions"] });
		},
		onError: (e) => setImportMsg(`Import failed: ${String(e)}`),
	});

	if (user.isPending) return <div className="p-4">Loading…</div>;
	if (user.error)
		return <div className="p-4 text-red-400">{String(user.error)}</div>;

	const u = user.data;
	if (!u) return <div className="p-4 text-red-400">No user found</div>;

	return (
		<div className="p-6 max-w-2xl">
			<h1 className="text-2xl font-semibold mb-4">User Switcher</h1>

			<div className="mb-6 p-4 bg-neutral-800 rounded-xl">
				<div className="text-sm text-neutral-400">Current user</div>
				<div className="text-xl font-medium mt-1">{u.name}</div>
			</div>

			<div className="mb-4">
				<label
					htmlFor="new-user-name"
					className="block text-sm text-neutral-400 mb-1"
				>
					Create new user
				</label>
				<div className="flex gap-2">
					<input
						id="new-user-name"
						value={newName}
						onChange={(e) => setNewName(e.target.value)}
						placeholder="User name"
						className="flex-1 px-3 py-2 bg-neutral-800 rounded-lg border border-neutral-700 focus:outline-none focus:border-neutral-500"
					/>
					<button
						type="button"
						onClick={() => {
							if (newName.trim()) createMut.mutate(newName.trim());
						}}
						disabled={createMut.isPending}
						className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50"
					>
						Create
					</button>
				</div>
			</div>

			<div className="mb-4">
				<label
					htmlFor="switch-user-id"
					className="block text-sm text-neutral-400 mb-1"
				>
					Switch user (enter user ID)
				</label>
				<div className="flex gap-2">
					<input
						id="switch-user-id"
						type="number"
						value={switchId}
						onChange={(e) => setSwitchId(e.target.value)}
						className="flex-1 px-3 py-2 bg-neutral-800 rounded-lg border border-neutral-700 focus:outline-none focus:border-neutral-500"
					/>
					<button
						type="button"
						onClick={() => switchMut.mutate(Number.parseInt(switchId, 10))}
						disabled={switchMut.isPending}
						className="px-4 py-2 bg-neutral-700 rounded-lg hover:bg-neutral-600"
					>
						Switch
					</button>
				</div>
			</div>

			<button
				type="button"
				onClick={() => deleteMut.mutate(u.id)}
				disabled={deleteMut.isPending}
				className="mb-4 px-4 py-2 text-red-400 hover:text-red-300"
			>
				Delete current user
			</button>

			<div className="mt-6 space-y-4">
				<div>
					<label
						htmlFor="export-path"
						className="block text-sm text-neutral-400 mb-1"
					>
						Export account to JSON file
					</label>
					<div className="flex gap-2">
						<input
							id="export-path"
							value={exportPath}
							onChange={(e) => setExportPath(e.target.value)}
							placeholder="~/yoube-export.json"
							className="flex-1 px-3 py-2 bg-neutral-800 rounded-lg border border-neutral-700 focus:outline-none focus:border-neutral-500"
						/>
						<button
							type="button"
							onClick={() => {
								setExportMsg(null);
								exportMut.mutate(exportPath);
							}}
							disabled={exportMut.isPending}
							className="px-4 py-2 bg-neutral-700 rounded-lg hover:bg-neutral-600 disabled:opacity-50"
						>
							Export
						</button>
					</div>
					{exportMsg && (
						<div className="mt-1 text-sm text-neutral-400">{exportMsg}</div>
					)}
				</div>

				<div>
					<label
						htmlFor="import-path"
						className="block text-sm text-neutral-400 mb-1"
					>
						Import account from JSON file
					</label>
					<div className="flex gap-2">
						<input
							id="import-path"
							value={importPath}
							onChange={(e) => setImportPath(e.target.value)}
							placeholder="~/yoube-export.json"
							className="flex-1 px-3 py-2 bg-neutral-800 rounded-lg border border-neutral-700 focus:outline-none focus:border-neutral-500"
						/>
						<button
							type="button"
							onClick={() => {
								setImportMsg(null);
								importMut.mutate(importPath);
							}}
							disabled={importMut.isPending}
							className="px-4 py-2 bg-neutral-700 rounded-lg hover:bg-neutral-600 disabled:opacity-50"
						>
							Import
						</button>
					</div>
					{importMsg && (
						<div className="mt-1 text-sm text-neutral-400">{importMsg}</div>
					)}
				</div>
			</div>
		</div>
	);
}
