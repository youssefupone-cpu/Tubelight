import { Link, Outlet, createRootRoute } from "@tanstack/react-router";

export const Route = createRootRoute({
	component: () => (
		<div className="h-full flex flex-col">
			<header className="h-12 border-b border-neutral-800 flex items-center px-4 gap-4">
				<Link to="/" className="font-semibold">
					yoube
				</Link>
			</header>
			<main className="flex-1 min-h-0">
				<Outlet />
			</main>
		</div>
	),
});
