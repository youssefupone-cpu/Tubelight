import { useSyncExternalStore } from "react";

export interface CapturedError {
	message: string;
	route: string;
	at: string;
}

let current: CapturedError | null = null;
const listeners = new Set<() => void>();

function emit() {
	for (const l of listeners) l();
}

export function captureError(message: string) {
	current = {
		message: message.slice(0, 2000),
		route:
			typeof window !== "undefined"
				? window.location.hash || window.location.pathname
				: "",
		at: new Date().toISOString(),
	};
	emit();
}

export function clearError() {
	current = null;
	emit();
}

function subscribe(fn: () => void) {
	listeners.add(fn);
	return () => {
		listeners.delete(fn);
	};
}

/** Register once (main.tsx): funnels uncaught errors into the banner. */
export function installGlobalErrorCapture() {
	if (typeof window === "undefined") return;
	window.addEventListener("error", (e) => {
		captureError(e.message || String(e.error ?? "unknown error"));
	});
	window.addEventListener("unhandledrejection", (e) => {
		captureError(
			e.reason instanceof Error ? e.reason.message : String(e.reason),
		);
	});
}

export function useCapturedError(): CapturedError | null {
	return useSyncExternalStore(
		subscribe,
		() => current,
		() => null,
	);
}
