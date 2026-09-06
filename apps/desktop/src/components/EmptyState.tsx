import type { LucideIcon } from "lucide-react";
import { Ghost } from "lucide-react";

export function EmptyState({
	icon: Icon = Ghost,
	title,
	hint,
}: {
	icon?: LucideIcon;
	title: string;
	hint?: string;
}) {
	return (
		<div className="m-4 rounded-xl border border-dashed border-neutral-800 bg-neutral-900/60 px-6 py-12 text-center">
			<Icon size={28} className="mx-auto text-neutral-600" />
			<div className="mt-3 text-sm font-medium text-neutral-300">{title}</div>
			{hint && <div className="mt-1 text-sm text-neutral-500">{hint}</div>}
		</div>
	);
}
