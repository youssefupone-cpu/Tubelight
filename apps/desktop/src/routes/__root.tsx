import { useQuery } from "@tanstack/react-query";
import {
	Link,
	Outlet,
	createRootRoute,
	useNavigate,
} from "@tanstack/react-router";
import { current_user } from "@yoube/contracts";
import { Download, Play, Search, Settings } from "lucide-react";
import { useState } from "react";
import { ReportBanner } from "../components/ReportBanner";
import { initialOf } from "../lib/format";
import { cn } from "../lib/utils";

function SearchBar() {
	const navigate = useNavigate();
	const [q, setQ] = useState("");
	return (
		<form
			className="flex-1 max-w-xl mx-auto"
			onSubmit={(e) => {
				e.preventDefault();
				const query = q.trim();
				if (query) navigate({ to: "/results", search: { q: query } });
			}}
		>
			<div className="flex items-center gap-2 bg-neutral-800/80 border border-neutral-700/60 rounded-full px-4 py-1.5 focus-within:border-neutral-500 transition-colors">
				<Search size={15} className="text-neutral-500 flex-shrink-0" />
				<input
					value={q}
					onChange={(e) => setQ(e.target.value)}
					placeholder="Search videos, channels…"
					aria-label="Search"
					className="flex-1 min-w-0 bg-transparent text-sm outline-none placeholder:text-neutral-600"
				/>
			</div>
		</form>
	);
}

function UserAvatar() {
	const user = useQuery({ queryKey: ["current-user"], queryFn: current_user });
	return (
		<Link
			to="/users"
			title={user.data ? `Signed in as ${user.data.name}` : "Users"}
			className="w-8 h-8 rounded-full bg-neutral-700 hover:bg-neutral-600 flex items-center justify-center text-[13px] font-semibold transition-colors"
		>
			{initialOf(user.data?.name ?? null)}
		</Link>
	);
}

const iconLink =
	"p-2 rounded-full text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors";

export const Route = createRootRoute({
	component: () => (
		<div className="h-full flex flex-col">
			<header className="h-14 shrink-0 border-b border-neutral-800/80 bg-[#0a0a0b]/95 backdrop-blur flex items-center px-4 gap-3">
				<Link to="/" className="flex items-center gap-2 font-bold text-[17px]">
					<span className="w-[26px] h-[26px] rounded-lg bg-[#e62117] flex items-center justify-center">
						<Play size={13} className="text-white fill-white ml-px" />
					</span>
					tubelight
				</Link>
				<SearchBar />
				<nav className="flex items-center gap-1 ml-auto">
					<Link
						to="/library"
						search={{ tab: "subscriptions" }}
						title="Library"
						activeProps={{ className: "text-white bg-neutral-800" }}
						className={cn(iconLink, "hidden sm:flex px-3 py-2 text-sm")}
					>
						Library
					</Link>
					<Link
						to="/downloads"
						title="Downloads"
						activeProps={{ className: "text-white bg-neutral-800" }}
						className={iconLink}
					>
						<Download size={17} />
					</Link>
					<Link
						to="/settings"
						search={{ tab: "general" }}
						title="Settings"
						activeProps={{ className: "text-white bg-neutral-800" }}
						className={iconLink}
					>
						<Settings size={17} />
					</Link>
					<UserAvatar />
				</nav>
			</header>
			<ReportBanner />
			<main className="flex-1 min-h-0">
				<Outlet />
			</main>
		</div>
	),
});
