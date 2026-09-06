// DEV MOCK DATA — browser-only preview dataset.
//
// Rationale: `src/bindings.ts` is hand-stubbed in this checkout (real
// tauri-specta output is generated only on GTK machines). Returning canned
// data here makes `pnpm dev` render the full UI in a plain browser for
// design review and screenshots. Production Tauri builds regenerate
// `bindings.ts` with real `invoke` calls, so none of this ships.
//
// Thumbnails come from picsum.photos seeds (stable, network-cached);
// playback uses the locally generated `/mock/sample.mp4` files (no network).

import type {
	AccountPlaylist,
	Branding,
	Channel,
	ChannelRef,
	Format,
	HistoryEntryView,
	Job,
	Playlist,
	Segment,
	UserProfile,
	Video,
	VideoSummary,
} from "@yoube/contracts";

const thumb = (seed: string) => `https://picsum.photos/seed/${seed}/640/360`;
const avatar = (seed: string) => `https://picsum.photos/seed/${seed}/96/96`;

export const MOCK_VIDEOS: VideoSummary[] = [
	{
		id: "dQw4w9WgXcQ",
		title: "100+ JavaScript Concepts you Need to Know",
		channel_id: "UCsBjURrPoezykLs9EqgamOA",
		channel_title: "Fireship",
		duration_s: 735,
		view_count: 2_400_000,
		upload_date: "20260312",
		thumbnail_url: thumb("yoube-js"),
	},
	{
		id: "9bZkp7q19f0",
		title: "Why Are 96,000,000 Black Balls on This Reservoir?",
		channel_id: "UCHnyfMqiRRG1u-2MsSQLbXA",
		channel_title: "Veritasium",
		duration_s: 1094,
		view_count: 18_200_000,
		upload_date: "20260228",
		thumbnail_url: thumb("yoube-balls"),
	},
	{
		id: "sNhhvQGsMEc",
		title: "The Fermi Paradox — Where Are All The Aliens?",
		channel_id: "UCsXVk37bltHxD1rDPwtNM8Q",
		channel_title: "Kurzgesagt – In a Nutshell",
		duration_s: 368,
		view_count: 52_700_000,
		upload_date: "20250115",
		thumbnail_url: thumb("yoube-fermi"),
	},
	{
		id: "jNQXAC9IVRw",
		title: "الدحيح | الذكاء الاصطناعي هل يفكر فعلا؟",
		channel_id: "UC7Eh8zHwPUmD8Q5dC2Qh6TA",
		channel_title: "الدحيح",
		duration_s: 1520,
		view_count: 3_100_000,
		upload_date: "20260301",
		thumbnail_url: thumb("yoube-daheeh"),
	},
	{
		id: "eKFTSSKCzWA",
		title: "I Built a PC Inside a Fish Tank (It Boils)",
		channel_id: "UCXuqSBlHAE6Xw-yeJA0Tunw",
		channel_title: "Linus Tech Tips",
		duration_s: 1187,
		view_count: 4_800_000,
		upload_date: "20260308",
		thumbnail_url: thumb("yoube-fishtank"),
	},
	{
		id: "WUvTyaaNkzM",
		title: "But what is a neural network?",
		channel_id: "UCYO_jab_esuFRV4b17AJt6A",
		channel_title: "3Blue1Brown",
		duration_s: 1139,
		view_count: 12_600_000,
		upload_date: "20241005",
		thumbnail_url: thumb("yoube-nn"),
	},
	{
		id: "RgKAFK5djSk",
		title: "The Untold History of the Transistor",
		channel_id: "UCHnyfMqiRRG1u-2MsSQLbXA",
		channel_title: "Veritasium",
		duration_s: 2345,
		view_count: 9_300_000,
		upload_date: "20260120",
		thumbnail_url: thumb("yoube-transistor"),
	},
	{
		id: "09R8_2nJtjg",
		title: "CSS in 100 Seconds (plus Grid and Flexbox)",
		channel_id: "UCsBjURrPoezykLs9EqgamOA",
		channel_title: "Fireship",
		duration_s: 228,
		view_count: 1_900_000,
		upload_date: "20260315",
		thumbnail_url: thumb("yoube-css"),
	},
	{
		id: "fJ9rUzIMcZQ",
		title: "The Egg — A Short Story",
		channel_id: "UCsXVk37bltHxD1rDPwtNM8Q",
		channel_title: "Kurzgesagt – In a Nutshell",
		duration_s: 469,
		view_count: 38_400_000,
		upload_date: "20230901",
		thumbnail_url: thumb("yoube-egg"),
	},
	{
		id: "kJQP7kiw5Fk",
		title: "Despacito — Official Music Video",
		channel_id: "UC-9-kyTW8ZkZNDHQJ6FgpwQ",
		channel_title: "Luis Fonsi",
		duration_s: 282,
		view_count: 8_600_000_000,
		upload_date: "20170112",
		thumbnail_url: thumb("yoube-despacito"),
	},
	{
		id: "60ItHLz5WEA",
		title: "Alan Watts — The Dream of Life (Remastered)",
		channel_id: "UCaBhY1uzpBV8nWpu6x0rJ8A",
		channel_title: "Tragedy & Hope",
		duration_s: 3120,
		view_count: 21_000_000,
		upload_date: "20200630",
		thumbnail_url: thumb("yoube-watts"),
	},
	{
		id: "M7lc1UVf-VE",
		title: "How Rust Ate JavaScript — Fireship Documentary",
		channel_id: "UCsBjURrPoezykLs9EqgamOA",
		channel_title: "Fireship",
		duration_s: 891,
		view_count: 980_000,
		upload_date: "20260318",
		thumbnail_url: thumb("yoube-rust"),
	},
];

export const MOCK_CHANNELS: Record<string, Channel> = {
	UCsBjURrPoezykLs9EqgamOA: {
		id: "UCsBjURrPoezykLs9EqgamOA",
		title: "Fireship",
		description:
			"High-intensity code tutorials and tech news to help you ship faster. New videos every week covering JavaScript, Rust, Python, and cloud.",
		thumb_url: avatar("ch-fireship"),
	},
	UCHnyfMqiRRG1u2MsSQLbXA: {
		id: "UCHnyfMqiRRG1u-2MsSQLbXA",
		title: "Veritasium",
		description:
			"An element of truth — videos about science, education, and anything curious in the world around us.",
		thumb_url: avatar("ch-veritasium"),
	},
};

export function mockChannel(id: string): Channel {
	return (
		MOCK_CHANNELS[id] ?? {
			id,
			title: `Channel ${id.slice(0, 6)}`,
			description:
				"A YouTube channel. Full about text loads in the desktop app.",
			thumb_url: avatar(`ch-${id.slice(0, 6)}`),
		}
	);
}

export function mockVideosForChannel(channelId: string): VideoSummary[] {
	const mine = MOCK_VIDEOS.filter((v) => v.channel_id === channelId);
	return mine.length > 0 ? mine : MOCK_VIDEOS.slice(0, 6);
}

export function searchMocks(q: string): VideoSummary[] {
	const tokens = q.toLowerCase().split(/\s+/).filter(Boolean);
	if (tokens.length === 0) return MOCK_VIDEOS;
	return MOCK_VIDEOS.filter((v) =>
		tokens.every((t) =>
			`${v.title} ${v.channel_title}`.toLowerCase().includes(t),
		),
	);
}

export function mockVideo(id: string): Video {
	const summary = MOCK_VIDEOS.find((v) => v.id === id) ?? MOCK_VIDEOS[0];
	const formats: Format[] = [
		{
			format_id: "22",
			ext: "mp4",
			url: "/mock/sample.mp4",
			resolution: "1280x720",
			fps: 30,
			vcodec: "avc1.64001F",
			acodec: "mp4a.40.2",
			filesize: 45_447_700,
			tbr: 1800,
			note: "720p",
		},
		{
			format_id: "18",
			ext: "mp4",
			url: "/mock/sample.mp4",
			resolution: "640x360",
			fps: 30,
			vcodec: "avc1.42001E",
			acodec: "mp4a.40.2",
			filesize: 18_200_000,
			tbr: 720,
			note: "360p",
		},
		{
			format_id: "140",
			ext: "m4a",
			url: "/mock/sample2.mp4",
			resolution: null,
			fps: null,
			vcodec: "none",
			acodec: "mp4a.40.2",
			filesize: 4_100_000,
			tbr: 128,
			note: "audio only",
		},
	];
	return { summary, formats };
}

export function mockPlaylist(id: string): Playlist {
	return {
		id,
		title: "Road trip mix",
		channel_id: MOCK_VIDEOS[0].channel_id,
		items: MOCK_VIDEOS.slice(0, 5),
	};
}

export const MOCK_USER: UserProfile = {
	id: 1,
	name: "youssef",
	avatar_path: null,
};

export const MOCK_SUBSCRIPTIONS: ChannelRef[] = [
	{
		id: "UCsBjURrPoezykLs9EqgamOA",
		title: "Fireship",
		thumb_url: avatar("ch-fireship"),
	},
	{
		id: "UCHnyfMqiRRG1u-2MsSQLbXA",
		title: "Veritasium",
		thumb_url: avatar("ch-veritasium"),
	},
	{
		id: "UCsXVk37bltHxD1rDPwtNM8Q",
		title: "Kurzgesagt – In a Nutshell",
		thumb_url: avatar("ch-kurz"),
	},
];

export const MOCK_PLAYLISTS: AccountPlaylist[] = [
	{
		id: 1,
		title: "Liked videos",
		description: null,
		is_watch_later: false,
		is_liked: true,
		count: 4,
	},
	{
		id: 2,
		title: "Watch later",
		description: null,
		is_watch_later: true,
		is_liked: false,
		count: 2,
	},
	{
		id: 3,
		title: "Road trip",
		description: "For the drive north",
		is_watch_later: false,
		is_liked: false,
		count: 5,
	},
	{
		id: 4,
		title: "To rewatch",
		description: null,
		is_watch_later: false,
		is_liked: false,
		count: 0,
	},
];

export function mockPlaylistItems(id: number): VideoSummary[] {
	if (id === 1) return MOCK_VIDEOS.slice(0, 4);
	if (id === 2) return MOCK_VIDEOS.slice(4, 6);
	if (id === 3) return MOCK_VIDEOS.slice(0, 5);
	return [];
}

export const MOCK_HISTORY: HistoryEntryView[] = MOCK_VIDEOS.slice(0, 6).map(
	(v, i) => ({
		video_id: v.id,
		title: v.title,
		channel_title: v.channel_title,
		duration_s: v.duration_s ?? null,
		thumb_url: v.thumbnail_url ?? null,
		watched_at: `2026-09-0${6 - i}T20:1${i}:00`,
		position_s: (i + 1) * 120,
	}),
);

export const MOCK_JOBS: Job[] = [
	{
		id: 3,
		video_id: "sNhhvQGsMEc",
		format_id: "22",
		dest_dir: "/home/youssef/Videos",
		state: "Running",
		progress_bytes: 27_100_000,
		total_bytes: 45_447_700,
		eta_s: 42,
		error: null,
	},
	{
		id: 4,
		video_id: "eKFTSSKCzWA",
		format_id: "18",
		dest_dir: "/home/youssef/Videos",
		state: "Queued",
		progress_bytes: 0,
		total_bytes: null,
		eta_s: null,
		error: null,
	},
	{
		id: 2,
		video_id: "9bZkp7q19f0",
		format_id: "22",
		dest_dir: "/home/youssef/Videos",
		state: "Done",
		progress_bytes: 45_447_700,
		total_bytes: 45_447_700,
		eta_s: null,
		error: null,
	},
	{
		id: 1,
		video_id: "kJQP7kiw5Fk",
		format_id: "140",
		dest_dir: "/home/youssef/Music",
		state: "Failed",
		progress_bytes: 900_000,
		total_bytes: 4_100_000,
		eta_s: null,
		error: "network timeout",
	},
];

export const MOCK_SEGMENTS: Segment[] = [
	{ category: "intro", start_s: 0, end_s: 12, uuid: "mock-intro-1" },
	{ category: "sponsor", start_s: 60, end_s: 95, uuid: "mock-sponsor-1" },
	{ category: "selfpromo", start_s: 400, end_s: 430, uuid: "mock-self-1" },
];

export const MOCK_BRANDING: Branding = {
	title: "100+ JavaScript Concepts (unclickbaited)",
	thumbnail_url: thumb("yoube-js-clean"),
};
