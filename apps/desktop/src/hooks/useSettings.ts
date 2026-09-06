import { useEffect, useState } from "react";

const PREFIX = "yoube.settings.";

function read<T>(key: string, fallback: T): T {
	try {
		const raw = localStorage.getItem(PREFIX + key);
		if (raw == null) return fallback;
		return JSON.parse(raw) as T;
	} catch {
		return fallback;
	}
}

/** Persisted UI setting backed by localStorage (namespaced `yoube.settings.*`).
 *
 * NOTE: the spec's Rust JSON-file `SettingsService`
 * (`app_data_dir/settings.json` + migrations) is a documented follow-up; the
 * frontend store mirrors the same keys so the migration is mechanical.
 */
export function useSetting<T>(key: string, fallback: T) {
	const [value, setValue] = useState<T>(() => read(key, fallback));
	useEffect(() => {
		try {
			localStorage.setItem(PREFIX + key, JSON.stringify(value));
		} catch {
			/* private mode — keep in memory */
		}
	}, [key, value]);
	return [value, setValue] as const;
}

export const DEFAULT_SKIP_CATEGORIES = [
	"sponsor",
	"intro",
	"outro",
	"selfpromo",
	"preview",
	"music_offtopic",
	"filler",
] as const;
